use chrono::TimeZone;

use super::*;


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
fn test_vec_to_blobhex() {
    let blob_data = vec![0u8, 0, 0, 0, 20, 228, 220, 207];
    let formatted = dbfmt(Some(blob_data));
    assert_eq!(formatted, "X'0000000014E4DCCF'");
}

#[test]
fn test_db_macro() {
    let name = Some("John's");
    let none: Option<&str> = None;
    let result = db!("The dog is {}, my dog is {}, my age is {}.", name, none, 42.3);
    // Note: format_value_inner escapes ' to '', so "John's" becomes "John''s"
    assert_eq!(result, "The dog is 'John''s', my dog is NULL, my age is 42.3.");
}

#[test]
fn test_db_macro_i32() {
    let result = db!("my age is {}.", 32i32);
    assert_eq!(result, "my age is 32.");
}

#[test]
fn test_db_macro_usize() {
    let result = db!("my age is {}.", &44usize);
    assert_eq!(result, "my age is 44.");
}

#[test]
fn test_db_macro_f32() {
    let result = db!("my age is {}.", 32.4f32);
    assert_eq!(result, "my age is 32.4.");
}

#[test]
fn test_db_macro_some_i32() {
    let age = Some(&32i32);
    let result = db!("my age is {}.", age);
    assert_eq!(result, "my age is 32.");
}

#[test]
fn test_db_macro_none() {
    let age:Option<i64> = None;
    let result = db!("my age is {}.", age);
    assert_eq!(result, "my age is NULL.");
}

#[test]
fn test_db_macro_strs() {
    let t1 = "Hi";
    let t2 = Some("my");
    let t3 = "name".to_string();
    let t4 = Some("is".to_string());
    let result = db!("{} {} {} {}", t1, t2, t3, t4);
    assert_eq!(result, "'Hi' 'my' 'name' 'is'");
}

#[test]
fn test_db_macro_vecs() {
    let t1 = vec![1u8, 2, 40, 100];
    let t2 = Some(vec![1u8,2,40,100]);
    let t3 = [1u8, 2, 40, 100].as_slice();
    let t4 = Some([1u8,2,40,100].as_slice());
    let result = db_nocomp!("{} {} {} {}", t1, t2, t3, t4);
    assert_eq!(result, "X'01022864' X'01022864' X'01022864' X'01022864'");
}

/// Helper: extract raw bytes from SQLite hex blob format: X'ABCD...'
fn decode_sqlite_blob_hex(s: &str) -> Vec<u8> {
    assert!(s.starts_with("X'") && s.ends_with("'"));

    let hex = &s[2..s.len() - 1];
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn format_blob_without_compression() {
    let data: Vec<u8> = vec![1, 2, 3, 4, 255, 0];

    let formatted = format_value_inner(&data, "", false);

    let decoded = decode_sqlite_blob_hex(&formatted);

    // No compression → bytes must be identical
    assert_eq!(decoded, data);
}

#[test]
fn format_blob_with_zstd_compression() {
    let data: Vec<u8> =
        b"The quick brown fox jumps over the lazy dog".to_vec();

    let formatted = format_value_inner(&data, "", true);

    let decoded = decode_sqlite_blob_hex(&formatted);

    // Must contain zstd magic header
    assert!(
        decoded.starts_with(ZSTD_MAGIC),
        "compressed blob must start with ZSTD magic header"
    );

    // Decompress and verify original payload
    let decompressed = maybe_decompress_blob(&decoded);

    assert_eq!(decompressed, data);
}

