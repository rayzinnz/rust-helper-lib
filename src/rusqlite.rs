use std::path::Path;

use anyhow::Result;
use rusqlite::{Connection, Error, OptionalExtension, Row};
use rusqlite::types::{FromSql, FromSqlResult, Value, ValueRef};

use crate::sql::maybe_decompress_blob;

/// returns the first column of the first row to i64, or none if no rows. Error on NULL or failed cast
pub fn query_to_i64(dbfilepath: &Path, sql:&str) -> Result<Option<i64>> {
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
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
pub fn query_to_string(dbfilepath: &Path, sql:&str) -> Result<Option<String>> {
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
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

pub fn query_single_row_to_tuple<T>(dbfilepath: &Path, sql:&str) -> Result<Option<T>, rusqlite::Error> 
where
    // The trait bound remains correct!
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
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

pub fn query_to_tuples_with_decompression<T>(
    dbfilepath: &Path,
    sql: &str,
) -> Result<Vec<T>, Error>
where
    T: TryFromValues,
{
    // 1. Open connection
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
	}

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

pub fn query_to_tuples<T>(dbfilepath: &Path, sql:&str) -> Result<Vec<T>, rusqlite::Error> 
where
    // T must implement TryFrom<&Row> for *any* lifetime 'r (HRTB remains crucial)
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
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

pub fn query_to_coltype<T>(dbfilepath: &Path, sql:&str) -> Result<Vec<T>, rusqlite::Error>
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
pub fn execute_batch(dbfilepath: &Path, sql:&str) -> Result<(), rusqlite::Error> 
{
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
	}
    
    conn.execute_batch(sql)
}

///execute sql to dbfilepath, return number of rows changed. Single statement only.
pub fn execute_return_changed_rows(dbfilepath: &Path, sql:&str) -> Result<usize, rusqlite::Error> 
{
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
	}
    
    conn.execute(sql, [])
}

///execute sql to dbfilepath, return last rowid. Can execute multiple statements within `sql` separated by ";"
pub fn execute_return_last_rowid(dbfilepath: &Path, sql:&str) -> Result<i64, rusqlite::Error> 
{
	let conn: Connection;
	if dbfilepath==Path::new("") {
		conn = Connection::open_in_memory()?;
	} else {
		conn = Connection::open(dbfilepath)?;
	}
    
    conn.execute_batch(sql)?;
    
    Ok(conn.last_insert_rowid())
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

#[cfg(test)]
mod tests {
	use std::fs;

	use chrono::{DateTime, Local, Timelike, Utc};

	use crate::{db, db_nocomp, db_format_arg, db_format_arg_no_compression, sql::{dbfmt_t, DbFormatWrapper, DbFormatWrapperNoCompression, FormatAny, FormatAnyNoCompression}};

	use super::*;

	#[test]
	fn test_query_to_i64() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT COUNT(*) FROM t;";
		let result = query_to_i64(dbfilepath, sql).unwrap();
		let expected: Option<i64> = Some(3);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_i64_no_rows() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		if !dbfilepath.exists() {
			panic!("dbfilepath not exists");
		}
		let sql = "SELECT c FROM t WHERE 1=2;";
		let result = query_to_i64(dbfilepath, sql).unwrap();
		let expected: Option<i64> = None;
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_i64_str() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT CAST(10 AS TEXT) AS c FROM t LIMIT 1;";
		let result = query_to_i64(dbfilepath, sql).unwrap();
		let expected: Option<i64> = Some(10);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_i64_null() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT NULL AS c FROM t LIMIT 1;";
		let result = query_to_i64(dbfilepath, sql);
		assert!(result.is_err());
	}

	#[test]
	fn test_query_single_row_to_tuple() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT 1 AS c1, 2 AS c2 FROM t LIMIT 1;";
		let result = query_single_row_to_tuple::<(i64,u8)>(dbfilepath, sql).unwrap();
		let expected: Option<(i64,u8)> = Some((1, 2));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_single_row_to_tuple_datetime_utc() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT datetime('now', 'utc');";
		let result = query_single_row_to_tuple::<(DateTime<Utc>,)>(dbfilepath, sql).unwrap();
		let now: DateTime<Utc> = Utc::now();
		let now = now.with_nanosecond(0).unwrap();
		let expected: Option<(DateTime<Utc>,)> = Some((now,));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_single_row_to_tuple_datetime_local() {
		// note; conversion to chrono::DateTime<Local> always
		//       assumes SELECTed date is in utc, so it will convert to local by adding offset.
		//       If SELECTed date is in localtime (e.g. SELECT datetime('now', 'localtime')) then the resulting date will be wrong (will add another offset to the localtime from sqlite).

		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT datetime('now', 'utc');";
		let result = query_single_row_to_tuple::<(DateTime<Local>,)>(dbfilepath, sql).unwrap();
		let now: DateTime<Local> = Local::now();
		let now = now.with_nanosecond(0).unwrap();
		let expected: Option<(DateTime<Local>,)> = Some((now,));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_single_row_to_tuple_no_rows() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT 1 AS c1, 2 AS c2 FROM t WHERE 1=2;";
		let result = query_single_row_to_tuple::<(i64,u8)>(dbfilepath, sql).unwrap();
		let expected: Option<(i64,u8)> = None;
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_tuples() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
		let result = query_to_tuples::<(i64,u8)>(dbfilepath, sql).unwrap();
		let mut expected: Vec<(i64,u8)> = Vec::new();
		expected.push((1,0));
		expected.push((2,0));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_coltype_int() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT c AS c2 FROM t LIMIT 2;";
		let result = query_to_coltype::<i64>(dbfilepath, sql).unwrap();
		let mut expected: Vec<i64> = Vec::new();
		expected.push(1);
		expected.push(2);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_tuples_conn() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let conn = Connection::open(dbfilepath).unwrap();
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
		let result = query_to_tuples_conn::<(i64,u8)>(&conn, sql).unwrap();
		let mut expected: Vec<(i64,u8)> = Vec::new();
		expected.push((1,0));
		expected.push((2,0));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_tuples_nullable() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 3;";
		let result = query_to_tuples::<(Option<i64>,u8)>(dbfilepath, sql).unwrap();
		let mut expected: Vec<(Option<i64>,u8)> = Vec::new();
		expected.push((Some(1),0));
		expected.push((Some(2),0));
		expected.push((None,0));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT 'string' FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).unwrap();
		let expected: Option<String> = Some(String::from("string"));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string_null() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT NULL FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).unwrap();
		let expected: Option<String> = None;
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string_int() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT 67 FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).unwrap();
		let expected: Option<String> = Some(String::from("67"));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string_real() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT 3.14159 FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).unwrap();
		let expected: Option<String> = Some(String::from("3.14159"));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string_blob() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT x'ff00e767' FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).unwrap();
		let expected: Option<String> = Some(String::from("ff00e767"));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_query_to_string_inmemory() {
		let sql = "SELECT 'string';";
		let result = query_to_string(Path::new(""), sql).unwrap();
		let expected: Option<String> = Some(String::from("string"));
		assert_eq!(result, expected);
	}

	#[test]
	fn test_execute_batch() {
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2); DELETE FROM t;";
		let result = execute_batch(Path::new(""), sql).unwrap();
		assert_eq!(result, ());
	}

	#[test]
	fn test_insert_vec() {
		let data: Vec<u8> = vec![1, 2, 3, 4];
		let sql = format!("CREATE TABLE t(c BLOB); INSERT INTO t VALUES ({}); DELETE FROM t;", dbfmt_t(&data));
		let result = execute_batch(Path::new(""), &sql).unwrap();
		assert_eq!(result, ());
	}

	#[test]
	fn test_execute_return_last_rowid() {
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2);INSERT INTO t VALUES (66);";
		let result = execute_return_last_rowid(Path::new(""), sql).unwrap();
		assert_eq!(result, 2);
	}

	#[test]
	fn test_execute_return_changed_rows() {
		let dbfilepath = std::env::temp_dir().join("helper_lib_test_temp_db.db");
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2); INSERT INTO t VALUES (66);";
		execute_batch(&dbfilepath, sql).unwrap();
		let result = execute_return_changed_rows(&dbfilepath, "UPDATE t SET c = 0").unwrap();
		//clean up temp db file
		if dbfilepath.exists() {
			fs::remove_file(dbfilepath).unwrap();
		}
		assert_eq!(result, 2);
	}

	#[test]
	fn query_to_tuples_handles_compressed_and_uncompressed_blobs() {
		// Persist DB to temp file so query_to_tuples can open it
		let dbfilepath = std::env::temp_dir().join("test_blobs.db");
		if dbfilepath.exists() {
			fs::remove_file(&dbfilepath).unwrap();
		}
		let conn = Connection::open(&dbfilepath).unwrap();

		conn.execute(
			"CREATE TABLE test_blobs (
				id INTEGER PRIMARY KEY,
				data BLOB NOT NULL
			)",
			[],
		)
		.unwrap();

		let raw_blob = b"raw blob data".to_vec();
		let compressed_blob = b"this blob is compressed".to_vec();

		// Insert uncompressed blob
		let sql_raw = db_nocomp!("INSERT INTO test_blobs (id, data) VALUES (1, {})",raw_blob);
		conn.execute(&sql_raw, []).unwrap();

		// Insert compressed blob
		let sql_compressed = db!("INSERT INTO test_blobs (id, data) VALUES (2, {})",compressed_blob);
		conn.execute(&sql_compressed, []).unwrap();

		conn.close().unwrap();

		//now select
		let rows:Vec<(i64, Vec<u8>)> = query_to_tuples_with_decompression(
			&dbfilepath,
			"SELECT id, data FROM test_blobs ORDER BY id",
		)
		.unwrap();

		assert_eq!(
			rows,
			vec![
				(1, raw_blob),
				(2, compressed_blob),
			]
		);

		if dbfilepath.exists() {
			fs::remove_file(&dbfilepath).unwrap();
		}
	}

	#[test]
	fn test_try_from_values_tuple() {
		use rusqlite::types::Value;

		let values = vec![
			Value::Integer(1),
			Value::Integer(0),
		];

		let result: (Option<i64>, u8) =
			<(Option<i64>, u8)>::try_from_values(values).unwrap();

		assert_eq!(result, (Some(1), 0));
	}

	#[test]
	fn test_query_to_values_then_tuple_nullable() {
		let dbfilepath = Path::new("./tests/resources/test.db");
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 3;";

		let result =
			query_to_tuples_with_decompression::<(Option<i64>, u8)>(dbfilepath, sql)
				.unwrap();

		let expected = vec![
			(Some(1), 0),
			(Some(2), 0),
			(None, 0),
		];

		assert_eq!(result, expected);
	}
}
