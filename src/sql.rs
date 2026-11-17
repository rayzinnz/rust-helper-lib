use std::any::Any;

use chrono::{DateTime, Local, Utc};
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
pub(crate) fn maybe_decompress_blob(bytes: &[u8]) -> Vec<u8> {
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


#[cfg(test)]
#[path = "./tests/sql_tests.rs"]
mod tests;
