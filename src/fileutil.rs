#![allow(dead_code)]
use regex::Regex;
use rmcp::ErrorData as McpError;
use std::fs;
use walkdir::DirEntry;

pub fn match_func_signature_in_file(
    name: &str,
    entry: &DirEntry,
    matches: &mut Vec<String>,
) -> Result<bool, McpError> {
    let Ok(content) = fs::read_to_string(entry.path()) else {
        return Ok(false);
    };

    let esc = regex::escape(name);
    // Python vs JS/TS patterns (handles multi-line params; anchored at start of line)
    let pattern = if entry.path().extension().and_then(|e| e.to_str()) == Some("py") {
        // allow optional "async" and decorators above; we only match the def line
        format!(
            r"(?ms)^[ \t]*(?:async[ \t]+)?def[ \t]+{}\s*\([^)]*?\)\s*:",
            esc
        )
    } else {
        // function decl OR arrow function; optional export/async
        format!(
            r"(?ms)^[ \t]*(?:export[ \t]+)?(?:async[ \t]+)?function[ \t]+{}\s*\([^)]*?\)\s*\{{|^[ \t]*(?:export[ \t]+)?(?:const|let|var)[ \t]+{}\s*=\s*\([^)]*?\)\s*=>[ \t]*\{{",
            esc, esc
        )
    };

    let re = Regex::new(&pattern).unwrap();
    let path_str = entry.path().display().to_string();

    // Precompute line starts
    let mut line_starts = Vec::with_capacity(256);
    line_starts.push(0);
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' {
            line_starts.push(i + 1);
        }
    }

    let byte_to_line_idx = |offset: usize| -> usize {
        match line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    };

    // let col_number_1based = |line_idx: usize, offset: usize| -> usize {
    //     let start = line_starts[line_idx];
    //     content[start..offset].chars().count() + 1
    // };

    for m in re.find_iter(&content) {
        let start = m.start();
        let end = m.end();

        let line_idx = byte_to_line_idx(start);
        let line_no = line_idx + 1;
        // let col_no = col_number_1based(line_idx, start);

        // Grab the whole matched text (multi-line signature included)
        let snippet = &content[start..end];
        let snippet_clean = snippet.trim_end();

        matches.push(format!("{}:{}: {}", path_str, line_no, snippet_clean));
    }

    Ok(true)
}

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
