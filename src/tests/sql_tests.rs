use chrono::{TimeZone, Timelike};

use super::*;
use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;

// A simple test struct to confirm non-string display formatting
#[derive(Debug, PartialEq)]
struct CustomType {
    id: u32,
}

// Implement Display for CustomType
impl Display for CustomType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "CustomID({})", self.id)
    }
}

// --- Tests for dbfmt_nullable ---

#[test]
fn test_optional_none_with_comparison_operator() {
    let input: Option<i32> = None;
    assert_eq!(dbfmt_comp(input, CompOp::NEq), " IS NOT NULL");
}

#[test]
fn test_optional_none() {
    let input: Option<i32> = None;
    assert_eq!(dbfmt(input), "NULL");
}

#[test]
fn test_optional_some_str_with_single_quote_with_comparison_operator() {
    let input: Option<&str> = Some("O'Brien's test");
    assert_eq!(dbfmt_comp(input, CompOp::LtEq), " <= 'O''Brien''s test'");
}

#[test]
fn test_optional_some_string_with_single_quote() {
    let input: Option<String> = Some("It's a test".to_string());
    assert_eq!(dbfmt(input), "'It''s a test'");
}

#[test]
fn test_optional_some_custom_type_display() {
    let input: Option<CustomType> = Some(CustomType { id: 99 });
    assert_eq!(dbfmt_comp(input, CompOp::Eq), " = CustomID(99)");
}

// --- Tests for dbfmt ---

#[test]
fn test_bare_str_with_single_quote() {
    let input: &str = "The customer's order";
    assert_eq!(dbfmt_t(&input), "'The customer''s order'");
}

#[test]
fn test_bare_string_with_single_quote_with_comparison_operator() {
    let input: String = "Manager's report".to_string();
    assert_eq!(dbfmt_t(&input), "'Manager''s report'");
}

#[test]
fn test_bare_i32() {
    let input: i32 = -500;
    assert_eq!(dbfmt_t(&input), "-500");
}

#[test]
fn test_bare_custom_type_display() {
    let input: CustomType = CustomType { id: 123 };
    assert_eq!(dbfmt_t(&input), "CustomID(123)");
}

#[test]
fn test_bare_datetime_utc() {
    let input: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 12, 25, 14, 30, 45).unwrap();
    assert_eq!(dbfmt_t(&input), "datetime('2023-12-25 14:30:45')");
}

#[test]
fn test_bare_datetime_local() {
    let input: DateTime<Local> = Local.with_ymd_and_hms(2023, 12, 25, 14, 30, 45).unwrap();
    //local times will be converted to utc in the database using the 'utc' modifier!
    assert_eq!(dbfmt_t(&input), "datetime('2023-12-25 14:30:45', 'utc')");
}

// MACRO tests

#[test]
/// Tests the new tuple-based syntax with two conditions.
fn test_new_tuple_based_syntax() {
    let result = where_sql!(
        "select c from t WHERE {} AND {}",
        ("c1", dbfmt_comp(Some(3), CompOp::Eq)), // 3 is an integer, so it should be formatted as an integer
        ("c2", dbfmt_comp::<String>(None, CompOp::NEq))
    );

    let expected = "select c from t WHERE c1 = 3 AND c2 IS NOT NULL";
    assert_eq!(result, expected);
}

#[test]
/// Tests the macro with mixed data types (string literal, integer, float) and multiple placeholders.
fn test_mixed_types_and_multiple_placeholders() {
    let result = where_sql!(
        "SELECT * FROM inventory WHERE {} AND location='warehouse' OR {}",
        ("product_id", dbfmt_comp(Some(101), CompOp::Eq)),
        ("price", dbfmt_comp(Some(49.99), CompOp::Eq))
    );

    let expected = "SELECT * FROM inventory WHERE product_id = 101 AND location='warehouse' OR price = 49.99";
    assert_eq!(result, expected);
}

#[test]
/// Tests the macro with only a single tuple and a single placeholder.
fn test_single_condition_and_placeholder() {
    let result = where_sql!(
        "SELECT id FROM orders WHERE {}",
        ("customer_id", dbfmt_comp(Some(5), CompOp::Gt))
    );

    let expected = "SELECT id FROM orders WHERE customer_id > 5";
    assert_eq!(result, expected);
}

#[test]
/// Tests macro when field names and values are passed as variables/expressions.
fn test_field_and_value_as_variables() {
    let field_name = "user_name";
    let value_data = Some("Alice");
    
    let result = where_sql!(
        "SELECT roles FROM access_table WHERE {} AND {}",
        (field_name, dbfmt_comp(value_data, CompOp::Eq)),
        ("status", dbfmt_comp(Some(2), CompOp::Lt))
    );
    
    let expected = "SELECT roles FROM access_table WHERE user_name = 'Alice' AND status < 2";
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_i64() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    if !dbfilepath.exists() {
        panic!("dbfilepath not exists");
    }
    let sql = "SELECT COUNT(*) FROM t;";
    let result = query_to_i64(&dbfilepath, sql).unwrap();
    let expected: Option<i64> = Some(3);
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_i64_no_rows() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    if !dbfilepath.exists() {
        panic!("dbfilepath not exists");
    }
    let sql = "SELECT c FROM t WHERE 1=2;";
    let result = query_to_i64(&dbfilepath, sql).unwrap();
    let expected: Option<i64> = None;
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_i64_str() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT CAST(10 AS TEXT) AS c FROM t LIMIT 1;";
    let result = query_to_i64(&dbfilepath, sql).unwrap();
    let expected: Option<i64> = Some(10);
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_i64_null() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT NULL AS c FROM t LIMIT 1;";
    let result = query_to_i64(&dbfilepath, sql);
    assert!(result.is_err());
}

#[test]
fn test_query_single_row_to_tuple() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT 1 AS c1, 2 AS c2 FROM t LIMIT 1;";
    let result = query_single_row_to_tuple::<(i64,u8)>(&dbfilepath, sql).unwrap();
    let expected: Option<(i64,u8)> = Some((1, 2));
    assert_eq!(result, expected);
}

#[test]
fn test_query_single_row_to_tuple_datetime_utc() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT datetime('now', 'utc');";
    let result = query_single_row_to_tuple::<(DateTime<Utc>,)>(&dbfilepath, sql).unwrap();
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

    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT datetime('now', 'utc');";
    let result = query_single_row_to_tuple::<(DateTime<Local>,)>(&dbfilepath, sql).unwrap();
    let now: DateTime<Local> = Local::now();
    let now = now.with_nanosecond(0).unwrap();
    let expected: Option<(DateTime<Local>,)> = Some((now,));
    assert_eq!(result, expected);
}

#[test]
fn test_query_single_row_to_tuple_no_rows() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT 1 AS c1, 2 AS c2 FROM t WHERE 1=2;";
    let result = query_single_row_to_tuple::<(i64,u8)>(&dbfilepath, sql).unwrap();
    let expected: Option<(i64,u8)> = None;
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_tuples() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
    let result = query_to_tuples::<(i64,u8)>(&dbfilepath, sql).unwrap();
    let mut expected: Vec<(i64,u8)> = Vec::new();
    expected.push((1,0));
    expected.push((2,0));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_tuples_conn() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let conn = Connection::open(&dbfilepath).unwrap();
    let sql = "SELECT c, 0 AS c2 FROM t LIMIT 2;";
    let result = query_to_tuples_conn::<(i64,u8)>(conn, sql).unwrap();
    let mut expected: Vec<(i64,u8)> = Vec::new();
    expected.push((1,0));
    expected.push((2,0));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_tuples_nullable() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT c, 0 AS c2 FROM t LIMIT 3;";
    let result = query_to_tuples::<(Option<i64>,u8)>(&dbfilepath, sql).unwrap();
    let mut expected: Vec<(Option<i64>,u8)> = Vec::new();
    expected.push((Some(1),0));
    expected.push((Some(2),0));
    expected.push((None,0));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_string() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT 'string' FROM t LIMIT 1;";
    let result = query_to_string(&dbfilepath, sql).unwrap();
    let expected: Option<String> = Some(String::from("string"));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_string_null() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT NULL FROM t LIMIT 1;";
    let result = query_to_string(&dbfilepath, sql).unwrap();
    let expected: Option<String> = None;
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_string_int() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT 67 FROM t LIMIT 1;";
    let result = query_to_string(&dbfilepath, sql).unwrap();
    let expected: Option<String> = Some(String::from("67"));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_string_real() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT 3.14159 FROM t LIMIT 1;";
    let result = query_to_string(&dbfilepath, sql).unwrap();
    let expected: Option<String> = Some(String::from("3.14159"));
    assert_eq!(result, expected);
}

#[test]
fn test_query_to_string_blob() {
    let dbfilepath = PathBuf::from("./tests/resources/test.db");
    let sql = "SELECT x'ff00e767' FROM t LIMIT 1;";
    let result = query_to_string(&dbfilepath, sql).unwrap();
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
