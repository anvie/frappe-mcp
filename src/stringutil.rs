/// Make a string into snake_case compliant and safe for Python identifiers.
/// For example, given this input: "Sales Invoice", it returns "sales_invoice".
pub fn to_snakec(name: &str) -> String {
    let name = name.trim();
    let mut result = String::with_capacity(name.len());
    let mut prev_was_underscore = false;
    for c in name.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            prev_was_underscore = false;
        } else if !prev_was_underscore {
            result.push('_');
            prev_was_underscore = true;
        }
    }
    // Remove trailing underscore if present
    if result.ends_with('_') {
        result.pop();
    }
    // Ensure it doesn't start with a digit
    if result.chars().next().map_or(false, |c| c.is_digit(10)) {
        result.insert(0, '_');
    }
    // If the result is empty (e.g., input was all non-alphanumeric), return a default name
    if result.is_empty() {
        return "default_name".to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_snakec() {
        let cases = vec![
            ("Sales Invoice", "sales_invoice"),
            ("123StartWithDigits", "_123startwithdigits"),
            ("Special@Chars!", "special_chars"),
            ("   Leading and Trailing   ", "leading_and_trailing"),
            ("MixedCASEInput", "mixedcaseinput"),
            ("", "default_name"),
            ("!!!", "default_name"),
            ("valid_name", "valid_name"),
            ("name-with-dashes", "name_with_dashes"),
            ("name.with.dots", "name_with_dots"),
        ];
        for (input, expected) in cases {
            assert_eq!(to_snakec(input), expected);
        }
    }
}
