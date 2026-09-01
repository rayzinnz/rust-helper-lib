/// Concatenates &str values into a new `String`.
///
/// # Examples
///
/// ```
/// use helper_lib::concat_str;
/// let name = "World";
/// let s = concat_str!("Hello, ", name, "!");
/// assert_eq!(s, "Hello, World!");
/// ```
#[macro_export]
macro_rules! concat_str {
    ($($s:expr),+ $(,)?) => {{
        let capacity = 0 $(+ $s.len())*;
        let mut result = String::with_capacity(capacity);
        $(
            result.push_str($s);
        )*
        result
    }};
}

/// Concatenates any number of string-like (&str, String, &String, Cow<'_, str>) values into a new `String`.
///
/// # Examples
///
/// ```
/// use helper_lib::concat_strref;
/// let name = String::from("World");
/// let s = concat_strref!("Hello, ", &name, "!");
/// assert_eq!(s, "Hello, World!");
/// ```
#[macro_export]
macro_rules! concat_strref {
    // concat 
    ($($s:expr),+ $(,)?) => {{
        let capacity = 0 $(
            + ::std::convert::AsRef::<str>::as_ref(&$s).len()
        )*;

        let mut result = String::with_capacity(capacity);

        $(
            result.push_str(::std::convert::AsRef::<str>::as_ref(&$s));
        )*

        result
    }};
}

/// Concatenates any number of anything implementing std::fmt::Display, including &str, String, 
/// integers, floats, booleans, and many custom types values into a new `String`.
///
/// # Examples
///
/// ```
/// use helper_lib::concat_any;
/// let s = concat_any!(
///     "count=",
///     42,
///     ", ok=",
///     true,
///     ", msg=",
///     String::from("hello")
/// );
/// 
/// assert_eq!(s, "count=42, ok=true, msg=hello");
/// ```
#[macro_export]
macro_rules! concat_any {
    // concat anything that implements Display to a String
    ($($x:expr),+ $(,)?) => {{
        use std::fmt::Write;

        let mut result = String::new();
        $(
            write!(&mut result, "{}", $x).unwrap();
        )*
        result
    }};
}

pub fn get_last_n_chars(s: &str, n: usize) -> String {
    s.chars().rev().take(n).collect::<String>().chars().rev().collect()
}

pub fn vec_u8_as_hex(data: &[u8], is_upper: bool, sep: &str) -> String {
    if sep.is_empty() && is_upper {
        data
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect()
    } else if is_upper {
        data
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<String>>()
            .join(sep)
    } else if sep.is_empty() {
        data
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    } else {
        data
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<String>>()
            .join(sep)
    }
}

pub fn string_from_utf16_as_vec_u8(utf16_data:&[u8]) -> String {
    String::from_utf16_lossy(
        &utf16_data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_concat_str() {
        let name = "World";
        let s = concat_str!("Hello, ", name, "!");
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn test_macro_concat_strref() {
        let name = String::from("World");
        let s = concat_strref!("Hello, ", name, "!");
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn test_macro_concat_any() {
        let s = concat_any!(
            "count=",
            42,
            ", ok=",
            true,
            ", msg=",
            String::from("hello")
        );

        assert_eq!(s, "count=42, ok=true, msg=hello");
    }

    #[test]
    fn test_get_last_n_chars() {
        let input: &str = "The customer's order";
        assert_eq!(get_last_n_chars(&input, 3), "der");
    }

    #[test]
    fn test_vec_u8_as_hex_nosep_lower() {
        let input: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(vec_u8_as_hex(&input, false, ""), "deadbeef");
    }

    #[test]
    fn test_vec_u8_as_hex_nosep_upper() {
        let input: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(vec_u8_as_hex(&input, true, ""), "DEADBEEF");
    }

    #[test]
    fn test_vec_u8_as_hex_sep_upper() {
        let input: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(vec_u8_as_hex(&input, true, "-"), "DE-AD-BE-EF");
    }

    #[test]
    fn test_string_from_utf16_as_vec_u8() {
        let input: Vec<u8> = vec![0x63, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x61, 0x00, 0x69, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x72, 0x00, 0x54, 0x00, 0x65, 0x00, 0x6D, 0x00, 0x70, 0x00, 0x65, 0x00, 0x72, 0x00, 0x61, 0x00, 0x74, 0x00];
        assert_eq!(string_from_utf16_as_vec_u8(&input), "containerTemperat");
    }
}