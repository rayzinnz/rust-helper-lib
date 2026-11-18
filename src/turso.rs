use chrono::{DateTime, Local, Utc};
use turso::{Builder, Connection, Row, Value};

use crate::sql::maybe_decompress_blob;

/// returns the first column of the first row to i64, or turso::Error if no rows or non-integer.
pub async fn query_to_i64(dbfilepath: &str, sql:&str) -> Result<Option<i64>, turso::Error> {
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.read_only(true)
			.build()
			.await?;
		conn = db.connect()?;
	}

	let mut rows = conn.query(sql, ()).await?;
	// match rows.next().await? {
	// 	Some(row) => Ok(row.get_value(0)?.as_integer().copied()),
	// 	None => Err(turso::Error::QueryReturnedNoRows)
	// }
	match rows.next().await? {
		//Some(row) => Ok(row.get_value(0)?.as_text().cloned()),
		Some(row) => match row.get_value(0)? {
			Value::Null => Ok(None),
			Value::Integer(v) => Ok(Some(v)),
			Value::Real(_) => Err(turso::Error::ConversionFailure(String::from("float returned, expect integer"))),
			Value::Text(_) => Err(turso::Error::ConversionFailure(String::from("string returned, expect integer"))),
			Value::Blob(_) => Err(turso::Error::ConversionFailure(String::from("blob returned, expect integer"))),
		},
		None => Err(turso::Error::QueryReturnedNoRows)
	}

}

/// returns the first column of the first row to String, or None if NULL. turso::Error on no rows
pub async fn query_to_string(dbfilepath: &str, sql:&str) -> Result<Option<String>, turso::Error> {
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.read_only(true)
			.build()
			.await?;
		conn = db.connect()?;
	}

	let mut rows = conn.query(sql, ()).await?;
	match rows.next().await? {
		//Some(row) => Ok(row.get_value(0)?.as_text().cloned()),
		Some(row) => match row.get_value(0)? {
			Value::Null => Ok(None),
			Value::Integer(v) => Ok(Some(v.to_string())),
			Value::Real(v) => Ok(Some(v.to_string())),
			Value::Text(v) => Ok(Some(v)),
			Value::Blob(v) => Ok(Some(hex::encode(maybe_decompress_blob(v.as_slice())))),
		},
		None => Err(turso::Error::QueryReturnedNoRows)
	}
}

pub async fn query_single_row_to_tuple<T>(dbfilepath: &str, sql:&str) -> Result<T, turso::Error> 
where
	T: TryFromValues,
{
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.read_only(true)
			.build()
			.await?;
		conn = db.connect()?;
	}

	let mut stmt = conn.prepare(sql).await?;
	let column_count = stmt.column_count();
	let row = stmt.query_row(()).await?;
	let values: Vec<Value> = row_to_values(&row, column_count);

	Ok(T::try_from_values(values)?)
}

pub async fn query_to_tuples_conn<T>(conn:&Connection, sql:&str) -> Result<Vec<T>, turso::Error> 
where
    T: TryFromValues,
{
	let mut stmt = conn.prepare(sql).await?;
	let column_count = stmt.column_count();
	let mut rows_converted = Vec::new();
	let mut rows = stmt.query(()).await?;
	while let Some(row) = rows.next().await? {
		// Convert Row -> Vec<Value> (blob-aware)
		let values: Vec<Value> = row_to_values(&row, column_count);
		// Convert Vec<Value> -> T
		rows_converted.push(T::try_from_values(values)?);
	}

	Ok(rows_converted)
}

pub async fn query_to_tuples<T>(
    dbfilepath: &str,
    sql: &str,
) -> Result<Vec<T>, turso::Error>
where
    T: TryFromValues,
{
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.read_only(true)
			.build()
			.await?;
		conn = db.connect()?;
	}

	Ok(query_to_tuples_conn(&conn, sql).await?)
}

pub async fn query_to_coltype<T>(dbfilepath: &str, sql:&str) -> Result<Vec<T>, turso::Error>
where
	T: Clone + std::fmt::Debug + FromValue,
{
	let v = query_to_tuples::<(T,)>(dbfilepath, sql).await?;

	let v2: Vec<T> = v.into_iter()
		.map(|(x,)| x)
		.collect();

	Ok(v2)
}

///execute sql to dbfilepath, void return. Can execute multiple statements within `sql` separated by ";"
pub async fn execute_batch(dbfilepath: &str, sql:&str) -> Result<(), turso::Error> 
{
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.build()
			.await?;
		conn = db.connect()?;
	}

	conn.execute_batch(sql).await
}

///execute sql to dbfilepath, return number of rows changed. Single statement only.
pub async fn execute_return_changed_rows(dbfilepath: &str, sql:&str) -> Result<u64, turso::Error> 
{
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.build()
			.await?;
		conn = db.connect()?;
	}

	conn.execute(sql, ()).await
}

///execute sql to dbfilepath, return last rowid. Can execute multiple statements within `sql` separated by ";"
pub async fn execute_return_last_rowid(dbfilepath: &str, sql:&str) -> Result<i64, turso::Error> 
{
	let conn: Connection;
	if dbfilepath.is_empty() {
		let db = Builder::new_local(":memory:").build().await?;
		conn = db.connect()?;
	} else {
		let db = Builder::new_local(dbfilepath)
			.build()
			.await?;
		conn = db.connect()?;
	}

	conn.execute_batch(sql).await?;

	Ok(conn.last_insert_rowid())
}

/// Convert a Vec<Value> into some tuple T
pub trait TryFromValues: Sized {
    fn try_from_values(values: Vec<Value>) -> Result<Self, turso::Error>;
}

macro_rules! tuple_try_from_values {
    ($($field:ident),*) => {
        impl<$($field,)*> TryFromValues for ($($field,)*)
        where
            $($field: FromValue,)*
        {
            fn try_from_values(values: Vec<Value>) -> Result<Self, turso::Error> {
                #[allow(unused_variables, unused_mut)]
                let mut iter = values.into_iter();

                $(
                    #[expect(non_snake_case)]
                    let $field: $field = FromValue::from_value(
                        iter.next()
                            .ok_or(turso::Error::ConversionFailure("in macro tuple_try_from_values!".to_string()))?
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
    fn from_value(v: Value) -> Result<Self, turso::Error>;
}
impl FromValue for u8 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as u8),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for u16 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as u16),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for u32 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as u32),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for u64 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as u64),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for i8 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as i8),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for i16 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as i16),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for i32 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i as i32),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for i64 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {Value::Integer(i) => Ok(i),_ => Err(turso::Error::ConversionFailure(String::from("Converting INTEGER failed"))),}}}
impl FromValue for f32 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {
    Value::Real(i) => Ok(i as f32),
    Value::Integer(i) => Ok(i as f32),
    _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to f32 failed.", v))),}}}
impl FromValue for f64 {fn from_value(v: Value) -> Result<Self, turso::Error> {match v {
    Value::Real(i) => Ok(i),
    Value::Integer(i) => Ok(i as f64),
    _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to f64 failed.", v))),}}}
impl FromValue for String {
    fn from_value(v: Value) -> Result<Self, turso::Error> {
        match v {
            Value::Text(s) => Ok(s),
            Value::Blob(b) => Ok(String::from_utf8_lossy(&b).to_string()),
            _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to string failed.", v))),
        }
    }
}
impl FromValue for Vec<u8> {
    fn from_value(v: Value) -> Result<Self, turso::Error> {
        match v {
            Value::Blob(b) => Ok(maybe_decompress_blob(&b)),
            Value::Text(s) => Ok(s.into_bytes()),
            _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to Vec<u8> failed.", v))),
        }
    }
}
impl FromValue for DateTime<Utc> {
    fn from_value(v: Value) -> Result<Self, turso::Error> {
        match v {
            Value::Integer(v) => match DateTime::from_timestamp_secs(v) {
				Some(v) => Ok(v),
				None => Err(turso::Error::ConversionFailure(format!("Converting {} to DateTime<Utc> failed.", v))),
			},
            Value::Text(v) => {
				let mut rfc3339_str = v.clone();
				//possible string values
				if rfc3339_str.len() == 10 {
					//yyyy-mm-dd
					rfc3339_str.push_str(" 00:00:00Z");
				} else {
					//yyyy-mm-dd HH:mm:ss
					rfc3339_str.push_str("Z");
				}
				match DateTime::parse_from_rfc3339(&rfc3339_str) {
					Ok(v) => Ok(v.to_utc()),
					Err(e) => Err(turso::Error::ConversionFailure(format!("Converting '{}' to DateTime<Utc> failed: {}", v, e))),
				}
			},
            Value::Real(_) => Err(turso::Error::ConversionFailure(format!("Conversion of float Julian day numbers to DateTime<Utc> is not supported."))),
            _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to DateTime<Utc> failed.", v))),
        }
    }
}
impl FromValue for DateTime<Local> {
    fn from_value(v: Value) -> Result<Self, turso::Error> {
        match v {
            Value::Integer(v) => match DateTime::from_timestamp_secs(v) {
				Some(v) => Ok(v.into()),
				None => Err(turso::Error::ConversionFailure(format!("Converting {} to DateTime<Local> failed.", v))),
			},
            Value::Text(v) => {
				let mut rfc3339_str = v.clone();
				//possible string values
				if rfc3339_str.len() == 10 {
					//yyyy-mm-dd
					rfc3339_str.push_str(" 00:00:00Z");
				} else {
					//yyyy-mm-dd HH:mm:ss
					rfc3339_str.push_str("Z");
				}
				match DateTime::parse_from_rfc3339(&rfc3339_str) {
					Ok(v) => Ok(v.to_utc().into()),
					Err(e) => Err(turso::Error::ConversionFailure(format!("Converting '{}' to DateTime<Local> failed: {}", v, e))),
				}
			},
            Value::Real(_) => Err(turso::Error::ConversionFailure(format!("Conversion of float Julian day numbers to DateTime<Local> is not supported."))),
            _ => Err(turso::Error::ConversionFailure(format!("Converting {:?} to DateTime<Local> failed.", v))),
        }
    }
}
impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn from_value(v: Value) -> Result<Self, turso::Error> {
        match v {
            Value::Null => Ok(None),
            other => T::from_value(other).map(Some),
        }
    }
}

fn row_to_values(row: &Row, column_count: usize) -> Vec<Value> {
    let mut out = Vec::with_capacity(column_count);

	for i in 0..column_count {
        let v = row.get_value(i).expect("could not row.get_value(i) in helperlib::sql::row_to_values()");
        out.push(v);
    }

    out
}

#[cfg(test)]
mod tests {
	use std::{fs, path::Path};
	use chrono::{DateTime, Local, Timelike, Utc};

	use crate::{db, db_nocomp, db_format_arg, db_format_arg_no_compression, sql::{dbfmt_t, DbFormatWrapper, DbFormatWrapperNoCompression, FormatAny, FormatAnyNoCompression}};

	use super::*;

	#[tokio::test]
	async fn test_query_to_i64() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT COUNT(*) FROM t;";
		let result = query_to_i64(dbfilepath, sql).await.unwrap();
		let expected: Option<i64> = Some(3);
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_i64_no_rows() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT c FROM t WHERE 1=2;";
		let result = query_to_i64(dbfilepath, sql).await;
		assert!(matches!(result, Err(turso::Error::QueryReturnedNoRows)));
	}

	#[tokio::test]
	async fn test_query_to_i64_str() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT CAST(10 AS TEXT) AS c FROM t LIMIT 1;";
		let result = query_to_i64(dbfilepath, sql).await;
		assert!(matches!(result, Err(turso::Error::ConversionFailure(_))));
	}

	#[tokio::test]
	async fn test_query_to_i64_null() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT NULL AS c FROM t LIMIT 1;";
		let result = query_to_i64(dbfilepath, sql).await.unwrap();
		let expected: Option<i64> = None;
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT 'string' FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).await.unwrap();
		let expected: Option<String> = Some(String::from("string"));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_null() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT NULL FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).await.unwrap();
		let expected: Option<String> = None;
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_int() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT 67 FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).await.unwrap();
		let expected: Option<String> = Some(String::from("67"));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_real() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT 3.14159 FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).await.unwrap();
		let expected: Option<String> = Some(String::from("3.14159"));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_blob() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT x'ff00e767' FROM t LIMIT 1;";
		let result = query_to_string(dbfilepath, sql).await.unwrap();
		let expected: Option<String> = Some(String::from("ff00e767"));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_inmemory() {
		let sql = "SELECT 'string';";
		let result = query_to_string("", sql).await.unwrap();
		let expected: Option<String> = Some(String::from("string"));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_string_no_rows() {
		let sql = "SELECT 1 WHERE 1=2;";
		let result = query_to_string("", sql).await;
		assert!(matches!(result, Err(turso::Error::QueryReturnedNoRows)));
	}

	#[tokio::test]
	async fn test_query_single_row_to_tuple() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT 1 AS c1, 2 AS c2 FROM t LIMIT 1;";
		let result = query_single_row_to_tuple::<(i64,u8)>(dbfilepath, sql).await.unwrap();
		let expected: (i64,u8) = (1, 2);
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_single_row_to_tuple_datetime_utc() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT datetime('now', 'utc');";
		let result = query_single_row_to_tuple::<(DateTime<Utc>,)>(dbfilepath, sql).await.unwrap();
		let now: DateTime<Utc> = Utc::now();
		let now = now.with_nanosecond(0).unwrap();
		let expected: (DateTime<Utc>,) = (now,);
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_single_row_to_tuple_datetime_local() {
		// note; conversion to chrono::DateTime<Local> always
		//       assumes SELECTed date is in utc, so it will convert to local by adding offset.
		//       If SELECTed date is in localtime (e.g. SELECT datetime('now', 'localtime')) then the resulting date will be wrong (will add another offset to the localtime from sqlite).

		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT datetime('now', 'utc');";
		let result = query_single_row_to_tuple::<(DateTime<Local>,)>(dbfilepath, sql).await.unwrap();
		let now: DateTime<Local> = Local::now();
		let now = now.with_nanosecond(0).unwrap();
		let expected: (DateTime<Local>,) = (now,);
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_single_row_to_tuple_no_rows() {
		let sql = "SELECT 1 WHERE 1=2;";
		let result = query_single_row_to_tuple::<(i64,u8)>("", sql).await;
		assert!(matches!(result, Err(turso::Error::QueryReturnedNoRows)));
	}

	#[tokio::test]
	async fn test_query_to_tuples() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
		let result = query_to_tuples::<(i64,u8)>(dbfilepath, sql).await.unwrap();
		let mut expected: Vec<(i64,u8)> = Vec::new();
		expected.push((1,0));
		expected.push((2,0));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_coltype_int() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT c AS c2 FROM t LIMIT 2;";
		let result = query_to_coltype::<i64>(dbfilepath, sql).await.unwrap();
		let mut expected: Vec<i64> = Vec::new();
		expected.push(1);
		expected.push(2);
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_tuples_conn() {
		let dbfilepath = "./tests/resources/test.db";
		let db = Builder::new_local(dbfilepath)
			.build()
			.await.unwrap();
		let conn = db.connect().unwrap();
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
		let result = query_to_tuples_conn::<(i64,u8)>(&conn, sql).await.unwrap();
		let mut expected: Vec<(i64,u8)> = Vec::new();
		expected.push((1,0));
		expected.push((2,0));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_query_to_tuples_nullable() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 3;";
		let result = query_to_tuples::<(Option<i64>,u8)>(dbfilepath, sql).await.unwrap();
		let mut expected: Vec<(Option<i64>,u8)> = Vec::new();
		expected.push((Some(1),0));
		expected.push((Some(2),0));
		expected.push((None,0));
		assert_eq!(result, expected);
	}

	#[tokio::test]
	async fn test_execute_batch() {
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2); DELETE FROM t;";
		let result = execute_batch("", sql).await.unwrap();
		assert_eq!(result, ());
	}

	#[tokio::test]
	async fn test_insert_vec() {
		let data: Vec<u8> = vec![1, 2, 3, 4];
		let sql = format!("CREATE TABLE t(c BLOB); INSERT INTO t VALUES ({}); DELETE FROM t;", dbfmt_t(&data));
		let result = execute_batch("", &sql).await.unwrap();
		assert_eq!(result, ());
	}

	#[tokio::test]
	async fn test_execute_return_last_rowid() {
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2);INSERT INTO t VALUES (66);";
		let result = execute_return_last_rowid("", sql).await.unwrap();
		assert_eq!(result, 2);
	}

	#[tokio::test]
	async fn test_execute_return_changed_rows() {
		let dbfilepath = std::env::temp_dir().join("helper_lib_test_temp_db.db");
		let dbfilepath = dbfilepath.to_string_lossy().to_string();
		// assert_eq!(dbfilepath, "".to_string());
		let sql = "CREATE TABLE t(c); INSERT INTO t VALUES (2); INSERT INTO t VALUES (66);";
		execute_batch(&dbfilepath, sql).await.unwrap();
		let result = execute_return_changed_rows(&dbfilepath, "UPDATE t SET c = 0").await.unwrap();
		//clean up temp db file
		if Path::new(&dbfilepath).exists() {
			fs::remove_file(dbfilepath).unwrap();
		}
		assert_eq!(result, 2);
	}

	#[tokio::test]
	async fn query_to_tuples_handles_compressed_and_uncompressed_blobs() {
		// Persist DB to temp file so query_to_tuples can open it
		let dbfilepath = std::env::temp_dir().join("test_blobs.db");
		let dbfilepath = dbfilepath.to_string_lossy().to_string();
		if Path::new(&dbfilepath).exists() {
			fs::remove_file(&dbfilepath).unwrap();
		}
		let raw_blob = b"raw blob data".to_vec();
		let compressed_blob = b"this blob is compressed".to_vec();
		{
			let db = Builder::new_local(&dbfilepath)
				.build()
				.await.unwrap();
			let conn = db.connect().unwrap();

			conn.execute(
				"CREATE TABLE test_blobs (
					id INTEGER PRIMARY KEY,
					data BLOB NOT NULL
				)",
				(),
			)
			.await.unwrap();

			// Insert uncompressed blob
			let sql_raw = db_nocomp!("INSERT INTO test_blobs (id, data) VALUES (1, {})",raw_blob);
			conn.execute(&sql_raw, ()).await.unwrap();

			// Insert compressed blob
			let sql_compressed = db!("INSERT INTO test_blobs (id, data) VALUES (2, {})",compressed_blob);
			conn.execute(&sql_compressed, ()).await.unwrap();
		}

		//now select
		let rows:Vec<(i64, Vec<u8>)> = query_to_tuples(
			&dbfilepath,
			"SELECT id, data FROM test_blobs ORDER BY id",
		)
		.await.unwrap();

		assert_eq!(
			rows,
			vec![
				(1, raw_blob),
				(2, compressed_blob),
			]
		);

		if Path::new(&dbfilepath).exists() {
			fs::remove_file(&dbfilepath).unwrap();
		}
	}

	#[tokio::test]
	async fn test_try_from_values_tuple() {
		use turso::value::Value;

		let values = vec![
			Value::Integer(1),
			Value::Integer(0),
		];

		let result: (Option<i64>, u8) =
			<(Option<i64>, u8)>::try_from_values(values).unwrap();

		assert_eq!(result, (Some(1), 0));
	}

	#[tokio::test]
	async fn test_query_to_values_then_tuple_nullable() {
		let dbfilepath = "./tests/resources/test.db";
		let sql = "SELECT c, 0 AS c2 FROM t LIMIT 3;";

		let result =
			query_to_tuples::<(Option<i64>, u8)>(dbfilepath, sql)
				.await.unwrap();

		let expected = vec![
			(Some(1), 0),
			(Some(2), 0),
			(None, 0),
		];

		assert_eq!(result, expected);
	}
}
