use regex::Regex;

pub fn match_to_string(string_to_search:&str, re:&Regex) -> Option<String> {
    match re.find(string_to_search) {
        Some(mat) => Some(mat.as_str().to_string()),
        None => None
    }
}

pub fn match_group_to_string(string_to_search:&str, re:&Regex, capturing_group:Option<usize>) -> Option<String> {
    re.captures(string_to_search)
        .and_then(|caps| caps.get(capturing_group.unwrap_or(0)))
        .map(|m| m.as_str().to_string())
}

pub fn matches_to_vec(string_to_search:&str, re:&Regex) -> Vec<String> {
    re.find_iter(string_to_search)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn matches_group_to_vec(string_to_search:&str, re:&Regex, capturing_group:Option<usize>) -> Vec<String> {
    re.captures_iter(string_to_search)
        .filter_map(|caps| Some(caps.get(capturing_group.unwrap_or(0))?.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
    fn test_match_to_string() {
        let re = Regex::new(r"\(.*\)").unwrap();
        let string_to_search = "![name](image/path/x.png)";
        let expected = String::from("(image/path/x.png)");
        assert_eq!(match_to_string(string_to_search, &re), Some(expected));
    }

	#[test]
    fn test_match_to_string_none() {
        let re = Regex::new(r"\(.*\)").unwrap();
        let string_to_search = "![name]image/path/x.png";
        assert_eq!(match_to_string(string_to_search, &re), None);
    }

	#[test]
    fn test_match_group_to_string() {
        let re = Regex::new(r"\(([^)]*)").unwrap();
        let string_to_search = "![name](image/path/x.png)";
        let expected = String::from("image/path/x.png");
        assert_eq!(match_group_to_string(string_to_search, &re, Some(1)), Some(expected));
    }

	#[test]
    fn test_match_group_to_string_none() {
        let re = Regex::new(r"\(([^)]*)").unwrap();
        let string_to_search = "![name]image/path/x.png";
        assert_eq!(match_group_to_string(string_to_search, &re, Some(1)), None);
    }

	#[test]
    fn test_match_group_to_string_nocap() {
        let re = Regex::new(r"\(.*").unwrap();
        let string_to_search = "![name](image/path/x.png)";
        let expected = String::from("(image/path/x.png)");
        let result = match_group_to_string(string_to_search, &re, None);
        assert_eq!(result, Some(expected));
    }

	#[test]
    fn test_matches_to_vec() {
        let re = Regex::new(r"!\[.*?\]\(.*?\)").unwrap();
        let string_to_search = "blah ![name](image/path/x.png) blah blah ![name](image/path/y.png) blah";
        let expected = vec![String::from("![name](image/path/x.png)"), String::from("![name](image/path/y.png)")];
        assert_eq!(matches_to_vec(string_to_search, &re), expected);
    }

	#[test]
    fn test_matches_to_vec_none() {
        let re = Regex::new(r"!\[.*?\]\(.*?\)").unwrap();
        let string_to_search = "blah [name](image/path/x.png) blah blah ![name]image/path/y.png) blah";
        let expected: Vec<String> = vec![];
        assert_eq!(matches_to_vec(string_to_search, &re), expected);
    }

	#[test]
    fn test_matches_group_to_vec() {
        let re = Regex::new(r"\(([^)]*)").unwrap();
        let string_to_search = "blah ![name](image/path/x.png) blah blah ![name](image/path/y.png) blah";
        let expected = vec![String::from("image/path/x.png"), String::from("image/path/y.png")];
        assert_eq!(matches_group_to_vec(string_to_search, &re, Some(1)), expected);
    }

}
