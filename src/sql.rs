use chrono::{DateTime, Local, Utc};
use rusqlite::{Connection, Error, OptionalExtension, Row};
use rusqlite::types::{ValueRef};
use std::{
    any::Any,
    convert::TryFrom,
    error::Error as StdError,
    fmt::Display,
    path::Path,
};

pub enum CompOp {
	Eq,
	NEq,
	Gt,
	GtEq,
	Lt,
	LtEq,
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


/// Private helper containing the core formatting logic for the inner value (T).
/// It handles the string escaping and default Display formatting.
fn format_value_inner<T>(value: &T, comparison_prefix: &str) -> String
where
    T: Display + Any + 'static,
{
    // Use the Any trait for runtime type checking
    let any_value = value as &dyn Any;

	// --- Check if the type is a String (&str or owned String) ---
    // If it is, apply escaping (' becomes '').
    if let Some(s) = any_value.downcast_ref::<&str>() {
        return format!("{}'{}'", comparison_prefix, s.replace("'", "''"));
    }

    if let Some(s) = any_value.downcast_ref::<String>() {
        return format!("{}'{}'", comparison_prefix, s.replace("'", "''"));
    }

    if let Some(s) = any_value.downcast_ref::<DateTime<Utc>>() {
        return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }

    if let Some(s) = any_value.downcast_ref::<DateTime<Local>>() {
        //convert local to utc. descision made to always store dates in utc, and use conversion functions for selecting and displaying to local time.
        return format!("{}datetime('{}', 'utc')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
        //return format!("{}datetime('{}')", comparison_prefix, s.format("%Y-%m-%d %H:%M:%S"));
    }

    // --- All other Display types (i32, f64, structs, etc.) ---
    format!("{}{}", comparison_prefix, value)
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
    T: Display + Any + 'static,
{
    format_value_inner(input, "")
}

/// Formats an optional value (Option<T>). This handles the None case.
///
/// This is used when the value might be missing (e.g., `let x: Option<i32> = None;`).
///
/// # Arguments
/// * `input` - A reference to the optional value.
pub fn dbfmt<T>(input: Option<T>) -> String
where
    T: Display + Any + 'static,
{
    match input {
        None => format!("NULL"),
        Some(value) => format_value_inner(&value, ""),
    }
}

/// as dbfmt, but prefixes a comparison operator. '=' for Some(), 'IS' for None()
pub fn dbfmt_comp<T>(input: Option<T>, comparison_operator: CompOp) -> String
where
    T: Display + Any + 'static,
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
			format_value_inner(&value, co)
		},
    }
}

/// returns the first column of the first row to i64, or none if no rows. Error on NULL or failed cast
pub fn query_to_i64(dbfilepath:&Path, sql:&str) -> Result<Option<i64>, Box<dyn StdError>> {
    let conn = Connection::open(&dbfilepath)?;
    
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

pub fn query_single_row_to_tuple<T>(dbfilepath:&Path, sql:&str) -> Result<Option<T>, rusqlite::Error> 
where
    // The trait bound remains correct!
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
    let conn = Connection::open(&dbfilepath)?;
    
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


pub fn query_to_tuples<T>(dbfilepath:&Path, sql:&str) -> Result<Vec<T>, rusqlite::Error> 
where
    // T must implement TryFrom<&Row> for *any* lifetime 'r (HRTB remains crucial)
    for<'r> T: TryFrom<
        &'r Row<'r>, 
        Error = Error 
    >
{
    let conn = Connection::open(&dbfilepath)?;
    
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

#[cfg(test)]
#[path = "./tests/sql_tests.rs"]
mod tests;
