use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use rusqlite::{Connection, Error, OptionalExtension, Row};
use rusqlite::types::{FromSql, FromSqlResult, Value, ValueRef};
use std::{
    any::Any,
    convert::TryFrom,
    path::Path,
};
use zstd::stream::{decode_all, encode_all};


/// Magic header to mark zstd-compressed blobs
const ZSTD_MAGIC: &[u8] = b"ZST\0";

pub enum CompOp {
	Eq,
	NEq,
	Gt,
	GtEq,
	Lt,
	LtEq,
}

// --- 1. The Autoref Specialization Wrapper & Traits ---
// A simple wrapper to control trait implementations
pub struct DbFormatWrapper<T>(pub T);
// High-priority trait for Options
pub trait FormatOption {
    fn format_db(self) -> String;
}
// Implemented on `&&DbFormatWrapper` (exact match, highest priority)
impl<T: Any + 'static> FormatOption for &&DbFormatWrapper<&Option<T>> {
    fn format_db(self) -> String {
        match self.0 {
            Some(v) => format_value_inner(v, "", true),
            None => String::from("NULL"),
        }
    }
}
// Low-priority fallback trait for direct values
pub trait FormatAny {
    fn format_db(self) -> String;
}
// Implemented on `&DbFormatWrapper` (requires auto-deref, lower priority)
impl<T: Any + 'static> FormatAny for &DbFormatWrapper<&T> {
    fn format_db(self) -> String {
        format_value_inner(self.0, "", true )
    }
}

// --- 2. The Macros ---
#[macro_export]
macro_rules! db_format_arg {
    // Intercept literal `None` immediately to prevent compiler type-inference 
    // errors (since `None` alone doesn't have a concrete inner type).
    (None) => {
        String::from("NULL")
    };
    // For all other typed variables or literals, use autoref dispatch.
    ($arg:expr) => {{
        // The double reference `&&` forces the compiler to try `FormatOption`
        // first. If it fails, it auto-derefs to `&` and hits `FormatAny`.
        (&&DbFormatWrapper(&$arg)).format_db()
    }};
}
#[macro_export]
macro_rules! db {
    // Zero-argument fallback
    ($fmt:expr $(,)?) => {
        format!($fmt)
    };
    // Format loop over arguments
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        format!($fmt, $( db_format_arg!($arg) ),*)
    };
}

//no compression
pub struct DbFormatWrapperNoCompression<T>(pub T);
// High-priority trait for Options
pub trait FormatOptionNoCompression {
    fn format_db(self) -> String;
}
// Implemented on `&&DbFormatWrapperNoCompression` (exact match, highest priority)
impl<T: Any + 'static> FormatOptionNoCompression for &&DbFormatWrapperNoCompression<&Option<T>> {
    fn format_db(self) -> String {
        match self.0 {
            Some(v) => format_value_inner(v, "", false),
            None => String::from("NULL"),
        }
    }
}
// Low-priority fallback trait for direct values
pub trait FormatAnyNoCompression {
    fn format_db(self) -> String;
}
// Implemented on `&DbFormatWrapperNoCompression` (requires auto-deref, lower priority)
impl<T: Any + 'static> FormatAnyNoCompression for &DbFormatWrapperNoCompression<&T> {
    fn format_db(self) -> String {
        format_value_inner(self.0, "", false)
    }
}
#[macro_export]
macro_rules! db_format_arg_no_compression {
    // Intercept literal `None` immediately to prevent compiler type-inference 
    // errors (since `None` alone doesn't have a concrete inner type).
    (None) => {
        String::from("NULL")
    };
    // For all other typed variables or literals, use autoref dispatch.
    ($arg:expr) => {{
        // The double reference `&&` forces the compiler to try `FormatOption`
        // first. If it fails, it auto-derefs to `&` and hits `FormatAny`.
        (&&DbFormatWrapperNoCompression(&$arg)).format_db()
    }};
}
#[macro_export]
macro_rules! db_nocomp {
    // Zero-argument fallback
    ($fmt:expr $(,)?) => {
        format!($fmt)
    };
    // Format loop over arguments
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        format!($fmt, $( db_format_arg_no_compression!($arg) ),*)
    };
}



/// Defines the `where_sql!` macro.
///
/// This macro takes a base SQL string as its first argument, followed by
/// an arbitrary number of (field, value) tuples. It is designed to replace
/// all `{}` placeholders in the base SQL string with the formatted
/// `field = value` expression from the corresponding tuple.
///
/// The number of (field, value) tuples MUST exactly match the number of
/// `{}` placeholders in the base SQL string.
///
/// # Arguments
/// * `$base_sql:literal`: The initial SQL string containing `{}` placeholders.
/// * `$( ($field:expr, $value:expr) ),*`: Repeating (field, value) tuples.
///   Fields and values must implement `ToString`.
///
/// # Example
/// `where_sql!("select c from t WHERE {} AND {}",("c1", dbfmt_comp(&Some(3), CompOp::Eq)),("c2", dbfmt_comp::<String>(&None, CompOp::NEq)));`
/// -> `"select c from t WHERE c1 = 3 AND c2 IS NOT NULL"`
#[macro_export]
macro_rules! where_sql {
    (
        // The base SQL string must be a literal string (e.g., "SELECT * FROM t WHERE {}")
        $base_sql:literal,
        // Capture repeating (field, value) tuples
        $( ($field:expr, $value:expr) ),*
    ) => {
        {
            // This expands to a single call to the standard `format!` macro.
            // 1. The first argument is the base SQL string literal.
            // 2. The subsequent arguments are a comma-separated list of dynamic
            //    expressions, each corresponding to a placeholder in the base string.
            format!(
                $base_sql,
                $(
                    // For each captured tuple, generate the replacement string: "field = value"
                    format!("{}{}", $field.to_string(), $value.to_string())
                ),*
            )
        }
    };
}

/// Decompress if zstd magic header is present
fn maybe_decompress_blob(bytes: &[u8]) -> Vec<u8> {
    if bytes.starts_with(ZSTD_MAGIC) {
        decode_all(&bytes[ZSTD_MAGIC.len()..])
            .expect("zstd decompression failed")
    } else {
        bytes.to_vec()
    }
}

/// Compress bytes using zstd and prefix with magic header
fn zstd_compress_with_magic(bytes: &[u8]) -> Vec<u8> {
    // Compression level:
    // 1–3: very fast
    // 5: balanced default
    // 9+: archival
    let compressed =
        encode_all(bytes, 5).expect("zstd compression failed");

    //prefix with magic bytes
    let mut out = Vec::with_capacity(ZSTD_MAGIC.len() + compressed.len());
    out.extend_from_slice(ZSTD_MAGIC);
    out.extend_from_slice(&compressed);
    out
}

/// Hex encoding helper (no allocations per byte)
fn sqlite_blob(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut hex = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(HEX[(b >> 4) as usize] as char);
        hex.push(HEX[(b & 0x0F) as usize] as char);
    }

    format!("X'{}'", hex)
}

/// Private helper containing the core formatting logic for the inner value (T).
/// It handles the string escaping and default Display formatting.
fn format_value_inner<T>(value: &T, comparison_prefix: &str, is_compress_blob: bool) -> String
where
    T: Any + 'static,
{
    // Use the Any trait for runtime type checking
    let any_value = value as &dyn Any;

	// --- Check if the type is a String (&str or owned String) ---
    // If it is, apply escaping (' becomes '').
    if let Some(s) = any_value.downcast_ref::<&str>() { return format!("{}'{}'", comparison_prefix, s.replace("'", "''")); }
    if let Some(s) = any_value.downcast_ref::<String>() { return format!("{}'{}'", comparison_prefix, s.replace("'", "''")); }
    if let Some(s) = any_value.downcast_ref::<&&str>() { return format!("{}'{}'", comparison_prefix, s.replace("'", "''")); }
    if let Some(s) = any_value.downcast_ref::<&String>() { return format!("{}'{}'", comparison_prefix, s.replace("'", "''")); }


    // --- Check if the type is a blob ---
    let format_blob = |bytes: &[u8]| {
        let bytes = if is_compress_blob {
            zstd_compress_with_magic(bytes)
        } else {
            bytes.to_vec()
        };
        format!("{}{}", comparison_prefix, sqlite_blob(&bytes))
    };

    // --- Check if the type is a Vec<u8> (blob) ---
    if let Some(blob) = any_value.downcast_ref::<Vec<u8>>() {
        // Convert bytes to hex string and format as X'hexstring'
        return format_blob(blob);
    }
    if let Some(blob) = any_value.downcast_ref::<&[u8]>() {
        // Convert bytes to hex string and format as X'hexstring'
        return format_blob(blob);
    }
    if let Some(blob) = any_value.downcast_ref::<&Vec<u8>>() {
        // Convert bytes to hex string and format as X'hexstring'
        return format_blob(blob);
    }
    if let Some(blob) = any_value.downcast_ref::<&&[u8]>() {
        // Convert bytes to hex string and format as X'hexstring'
        return format_blob(blob);
    }

    if let Some(s) = any_value.downcast_ref::<DateTime<Utc>>() {
        return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(s) = any_value.downcast_ref::<DateTime<Local>>() {
        //convert local to utc. descision made to always store dates in utc, and use conversion functions for selecting and displaying to local time.
        return format!("{}datetime('{}', 'utc')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
        //return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(s) = any_value.downcast_ref::<&DateTime<Utc>>() {
        return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(s) = any_value.downcast_ref::<&DateTime<Local>>() {
        //convert local to utc. descision made to always store dates in utc, and use conversion functions for selecting and displaying to local time.
        return format!("{}datetime('{}', 'utc')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
        //return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(v) = any_value.downcast_ref::<usize>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<i8>()   { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<i16>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<i32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<i64>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<i128>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<u8>()   { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<u16>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<u32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<u64>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<u128>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<f32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<f64>()  { return format!("{}{}", comparison_prefix, v); }

    if let Some(v) = any_value.downcast_ref::<&usize>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&i8>()   { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&i16>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&i32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&i64>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&i128>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&u8>()   { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&u16>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&u32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&u64>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&u128>() { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&f32>()  { return format!("{}{}", comparison_prefix, v); }
    if let Some(v) = any_value.downcast_ref::<&f64>()  { return format!("{}{}", comparison_prefix, v); }
    
    panic!("Unsupported type passed to format_value_inner");

    // // --- All other Display types (i32, f64, structs, etc.) ---
    // format!("{}{:?}", comparison_prefix, value)
}

// --- Public API Functions ---

/// Formats a bare value (T). Since the value is not an Option, it cannot be None.
///
/// This is used when you know the value is present (e.g., `let x = 42;`).
///
/// # Arguments
/// * `input` - A reference to the bare value.
pub fn dbfmt_t<T>(input: &T) -> String
where
    T: Any + 'static,
{
    format_value_inner(input, "", false)
}
pub fn dbfmt_t_comp<T>(input: &T) -> String
where
    T: Any + 'static,
{
    format_value_inner(input, "", true)
}

/// Formats an optional value (Option<T>). This handles the None case.
///
/// This is used when the value might be missing (e.g., `let x: Option<i32> = None;`).
///
/// # Arguments
/// * `input` - A reference to the optional value.
pub fn dbfmt<T>(input: Option<T>) -> String
where
    T: Any + 'static,
{
    match input {
        None => format!("NULL"),
        Some(value) => format_value_inner(&value, "", false),
    }
}

/// as dbfmt, but prefixes a comparison operator. '=' for Some(), 'IS' for None()
pub fn dbfmt_comp<T>(input: Option<T>, comparison_operator: CompOp) -> String
where
    T: Any + 'static,
{
    match input {
        None => {
			let co = match comparison_operator {
				CompOp::NEq => " IS NOT ",
				_ => " IS ",
			};
			format!("{}NULL", co)
		},
        Some(value) => {
			let co = match comparison_operator {
				CompOp::Eq => " = ",
				CompOp::NEq => " <> ",
				CompOp::Gt => " > ",
				CompOp::GtEq => " >= ",
				CompOp::Lt => " < ",
				CompOp::LtEq => " <= ",
			};
			format_value_inner(&value, co, false)
		},
    }
}

/// returns the first column of the first row to i64, or none if no rows. Error on NULL or failed cast
pub fn query_to_i64(dbfilepath:&Path, sql:&str) -> Result<Option<i64>> {
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    let result: Option<i64> = conn.query_row(sql, [], |row| {
        let value_ref = row.get_ref(0)?;

        let converted_value: i64 = match value_ref {
            // 1. INTEGER: Direct conversion
            ValueRef::Integer(i) => i,
            
            // 2. REAL: Convert to i64 by truncation (standard Rust f64 as i64)
            ValueRef::Real(f) => f as i64, 
            
            // 3. TEXT: Attempt to parse the string into an i64
            ValueRef::Text(bytes) => {
                // Convert the byte slice to a UTF-8 string, then parse
                let s = std::str::from_utf8(bytes)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
                
                s.parse::<i64>()
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
            }
            
            // 4. NULL: Handle as an error within the row closure (or you could return a default)
            ValueRef::Null => {
                return Err(rusqlite::Error::InvalidColumnType(0, String::from("NULL not an integer"), rusqlite::types::Type::Null));
            }
            
            // 5. BLOB: Cannot convert arbitrary binary data to i64
            ValueRef::Blob(_) => {
                return Err(rusqlite::Error::InvalidColumnType(0, String::from("BLOB not an integer"), rusqlite::types::Type::Blob));
            }
        };

        Ok(converted_value)
    }).optional()?;

    return Ok(result);
}

/// returns the first column of the first row to String, or None if NULL. Error on no rows or failed cast
pub fn query_to_string(dbfilepath:&Path, sql:&str) -> Result<Option<String>> {
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    // 2. Execute the query using query_row
    let result = conn.query_row(
        sql,
        [], // No parameters for this example, use `params!` or `&[]` for bind parameters
        |row| {
            // This closure maps a single row to the desired output.
            // We use get_raw(0) to check for NULL before attempting to convert to String.
            match row.get_ref(0)? {
                ValueRef::Null => Ok(None),
                // For INTEGER and REAL, use format! to convert to String without relying 
                // on the strict FromSql<String> implementation.
                ValueRef::Integer(i) => Ok(Some(format!("{}", i))),
                ValueRef::Real(f) => Ok(Some(format!("{}", f))),
                // BLOB: Convert byte slice to a hexadecimal String.
                ValueRef::Blob(bytes) => {
                    // Use the hex crate to encode the bytes into a lowercase hex string
                    Ok(Some(hex::encode(maybe_decompress_blob(bytes))))
                },
                // If it's Text, safely convert the byte slice to a String.
                ValueRef::Text(bytes) => {
                    // let formatted_string: String = row.get(0)?;
                    let formatted_string: String = String::from_utf8_lossy(bytes).to_string();
                    Ok(Some(formatted_string))
                }
            }
        },
    )?;

    // 3. Handle the result from query_row
    Ok(result)
}

pub fn query_single_row_to_tuple<T>(dbfilepath:&Path, sql:&str) -> Result<Option<T>, rusqlite::Error> 
where
    // The trait bound remains correct!
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    // 1. Use query_map instead of query_row
    let mut stmt = conn.prepare(sql)?;
    let result_iter = stmt.query_map([], |row| T::try_from(row));

    // 2. Map the MappedRows into a single T
    let result: Result<T, Error> = match result_iter {
        Ok(mut rows) => {
            // Get the first item from the iterator
            if let Some(row_result) = rows.next() {
                // If we get an item, return its result
                row_result
            } else {
                // If there are no items, simulate the "No Rows" error
                // This will be caught by the unwrap_or_else block below
                Err(Error::QueryReturnedNoRows)
            }
        }
        // If query_map itself fails (e.g., bad SQL), propagate that error
        Err(e) => Err(e),
    };
    
    // 3. Handle the result to return Option<T>
    match result {
        // If we successfully got a row
        Ok(t) => Ok(Some(t)),
        
        // If we got the specific "No Rows" error, return None
        Err(Error::QueryReturnedNoRows) => Ok(None),
        
        // If we got any other error (e.g., SQL error, I/O error), propagate it
        Err(e) => Err(e),
    }
}

/// for getting compressed data with using query_to_tuples(), and default FromSql things.
/// e.g. `let result = query_to_tuples::<(Option<i64>, MaybeDecompressed<Vec<u8>>)>(&dbpath, sql)?;`
pub struct MaybeDecompressed<T>(pub T);
impl<T> FromSql for MaybeDecompressed<T>
where
    T: FromSql,
{
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(b) => {
                // your custom logic
                let decompressed = maybe_decompress_blob(b);

                // IMPORTANT: feed it back as a ValueRef
                let vref = ValueRef::Blob(&decompressed);
                T::column_result(vref).map(MaybeDecompressed)
            }
            other => T::column_result(other).map(MaybeDecompressed),
        }
    }
}

/// Convert a Vec<Value> into some tuple T
pub trait TryFromValues: Sized {
    fn try_from_values(values: Vec<Value>) -> Result<Self, Error>;
}

macro_rules! tuple_try_from_values {
    ($($field:ident),*) => {
        impl<$($field,)*> TryFromValues for ($($field,)*)
        where
            $($field: FromValue,)*
        {
            fn try_from_values(values: Vec<Value>) -> Result<Self, Error> {
                #[allow(unused_variables, unused_mut)]
                let mut iter = values.into_iter();

                $(
                    #[expect(non_snake_case)]
                    let $field: $field = FromValue::from_value(
                        iter.next()
                            .ok_or(Error::InvalidColumnIndex(0))?
                    )?;
                )*

                Ok(($($field,)*))
            }
        }
    };
}

macro_rules! tuples_try_from_values {
    () => {
        tuple_try_from_values!();
    };
    ($first:ident $(, $rest:ident)*) => {
        tuple_try_from_values!($first $(, $rest)*);
        tuples_try_from_values!($($rest),*);
    };
}

// Match rusqlite arity
tuples_try_from_values!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// Local trait — satisfies orphan rules
pub trait FromValue: Sized {
    fn from_value(v: Value) -> Result<Self, Error>;
}
impl FromValue for u8 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as u8),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for u16 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as u16),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for u32 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as u32),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for u64 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as u64),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for i8 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as i8),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for i16 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as i16),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for i32 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i as i32),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for i64 {fn from_value(v: Value) -> Result<Self, Error> {match v {Value::Integer(i) => Ok(i),_ => Err(Error::InvalidColumnType(0,"INTEGER".into(),v.data_type(),)),}}}
impl FromValue for f32 {fn from_value(v: Value) -> Result<Self, Error> {match v {
    Value::Real(i) => Ok(i as f32),
    Value::Integer(i) => Ok(i as f32),
    _ => Err(Error::InvalidColumnType(0,"REAL".into(),v.data_type(),)),}}}
impl FromValue for f64 {fn from_value(v: Value) -> Result<Self, Error> {match v {
    Value::Real(i) => Ok(i),
    Value::Integer(i) => Ok(i as f64),
    _ => Err(Error::InvalidColumnType(0,"REAL".into(),v.data_type(),)),}}}
impl FromValue for String {
    fn from_value(v: Value) -> Result<Self, Error> {
        match v {
            Value::Text(s) => Ok(s),
            Value::Blob(b) => String::from_utf8(b).map_err(|e| {
                Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            }),
            _ => Err(Error::InvalidColumnType(
                0,
                "TEXT".into(),
                v.data_type(),
            )),
        }
    }
}
impl FromValue for Vec<u8> {
    fn from_value(v: Value) -> Result<Self, Error> {
        match v {
            Value::Blob(b) => Ok(b),
            Value::Text(s) => Ok(s.into_bytes()),
            _ => Err(Error::InvalidColumnType(
                0,
                "BLOB".into(),
                v.data_type(),
            )),
        }
    }
}
impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn from_value(v: Value) -> Result<Self, Error> {
        match v {
            Value::Null => Ok(None),
            other => T::from_value(other).map(Some),
        }
    }
}

fn row_to_values(row: &rusqlite::Row<'_>, column_count: usize) -> Vec<Value> {
    let mut out = Vec::with_capacity(column_count);

    for i in 0..column_count {
        let v = match row.get_ref(i).expect("could not row.get_ref(i) in helperlib::sql::row_to_values()") {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(i) => Value::Integer(i),
            ValueRef::Real(f) => Value::Real(f),
            ValueRef::Text(t) => Value::Text(String::from_utf8(t.to_vec()).unwrap()),
            ValueRef::Blob(b) => Value::Blob(maybe_decompress_blob(b)),
        };
        out.push(v);
    }

    out
}

pub fn query_to_tuples_with_decompression<T>(
    dbfilepath: &Path,
    sql: &str,
) -> Result<Vec<T>, Error>
where
    T: TryFromValues,
{
    // 1. Open connection
    let conn = if dbfilepath == Path::new("") {
        Connection::open_in_memory()?
    } else {
        Connection::open(dbfilepath)?
    };

    // 2. Prepare statement
    let mut stmt = conn.prepare(sql)?;
    
    // 3. Query and map rows
    let column_count = stmt.column_count();
    let rows = stmt.query_map([], |row: &Row<'_>| {
        // Convert Row -> Vec<Value> (blob-aware)
        let values: Vec<Value> = row_to_values(row, column_count);

        // Convert Vec<Value> -> T
        T::try_from_values(values)
    })?;

    // 4. Collect results
    let result: Result<Vec<T>, Error> = rows.collect();

    result
}

pub fn query_to_tuples<T>(dbfilepath:&Path, sql:&str) -> Result<Vec<T>, rusqlite::Error> 
where
    // T must implement TryFrom<&Row> for *any* lifetime 'r (HRTB remains crucial)
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    // 1. Prepare the SQL statement.
    let mut stmt = conn.prepare(sql)?;
    
    // 2. Use query_map to iterate and apply the conversion closure to every row.
    let rows_result = stmt.query_map([], |row| {
        // The closure uses your TryFrom constraint
        T::try_from(row)
    })?; // The first '?' handles errors during statement execution (e.g., bad SQL)

    // 3. Collect the MappedRows iterator.
    // The inner iterator yields Result<T, Error>. 
    // .collect() collects these into a Result<Vec<T>, Error>.
    let result_vec: Result<Vec<T>, Error> = rows_result
        .collect();
    
    // 4. Return the result. The '?' operator is often implicitly done 
    // if using the fully expressive method chaining, but here we return the Result<Vec<T>, Error>.
    result_vec
}

pub fn query_to_coltype<T>(dbfilepath:&Path, sql:&str) -> Result<Vec<T>, rusqlite::Error>
where
    T: Clone + std::fmt::Debug + FromSql,
{
    let v = query_to_tuples::<(T,)>(dbfilepath, sql)?;
    
    let v2: Vec<T> = v.into_iter()
        .map(|(x,)| x)
        .collect();

    Ok(v2)
}

pub fn query_to_tuples_conn<T>(conn:&Connection, sql:&str) -> Result<Vec<T>, rusqlite::Error> 
where
    // T must implement TryFrom<&Row> for *any* lifetime 'r (HRTB remains crucial)
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
    // 1. Prepare the SQL statement.
    let mut stmt = conn.prepare(sql)?;
    
    // 2. Use query_map to iterate and apply the conversion closure to every row.
    let rows_result = stmt.query_map([], |row| {
        // The closure uses your TryFrom constraint
        T::try_from(row)
    })?; // The first '?' handles errors during statement execution (e.g., bad SQL)

    // 3. Collect the MappedRows iterator.
    // The inner iterator yields Result<T, Error>. 
    // .collect() collects these into a Result<Vec<T>, Error>.
    let result_vec: Result<Vec<T>, Error> = rows_result
        .collect();
    
    // 4. Return the result. The '?' operator is often implicitly done 
    // if using the fully expressive method chaining, but here we return the Result<Vec<T>, Error>.
    result_vec
}

///execute sql to dbfilepath, void return. Can execute multiple statements within `sql` separated by ";"
pub fn execute_batch(dbfilepath:&Path, sql:&str) -> Result<(), rusqlite::Error> 
{
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    conn.execute_batch(sql)
}

///execute sql to dbfilepath, return number of rows changed. Single statement only.
pub fn execute_return_changed_rows(dbfilepath:&Path, sql:&str) -> Result<usize, rusqlite::Error> 
{
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    conn.execute(sql, [])
}

///execute sql to dbfilepath, return last rowid. Can execute multiple statements within `sql` separated by ";"
pub fn execute_return_last_rowid(dbfilepath:&Path, sql:&str) -> Result<i64, rusqlite::Error> 
{
    let conn: Connection;
    if dbfilepath == Path::new("") {
        conn = Connection::open_in_memory()?;
    } else {
        conn = Connection::open(&dbfilepath)?;
    }
    
    conn.execute_batch(sql)?;
    
    Ok(conn.last_insert_rowid())
}


#[cfg(test)]
#[path = "./tests/sql_tests.rs"]
mod tests;
