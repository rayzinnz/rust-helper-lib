pub fn get_last_n_chars(s: &str, n: usize) -> String {
    s.chars().rev().take(n).collect::<String>().chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_n_chars() {
        let input: &str = "The customer's order";
        assert_eq!(get_last_n_chars(&input, 3), "der");
    }

}