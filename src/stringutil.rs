// Copyright (C) 2025 Nuwaira
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Nuwaira.
// The intellectual and technical concepts contained
// herein are proprietary to Nuwaira
// and are protected by trade secret or copyright law.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Nuwaira.

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

/// Ubah teks apa pun menjadi CamelCase yang valid untuk nama kelas Python.
/// - Pisahkan pada karakter non-alfanumerik
/// - Kapitalisasi setiap kata (CapWords)
/// - Prefix "_" bila hasil diawali digit
/// - Kembalikan "_" bila tidak ada karakter alfanumerik
pub fn to_pascalc(input: &str) -> String {
    // Kumpulkan kata-kata yang berisi alfanumerik (Unicode-aware)
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();

    for ch in input.chars() {
        if ch.is_alphanumeric() {
            cur.push(ch);
        } else if !cur.is_empty() {
            words.push(cur);
            cur = String::new();
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }

    // Jika tidak ada kata alfanumerik -> "_"
    if words.is_empty() {
        return "_".to_string();
    }

    // // CapWords: huruf pertama Upper, sisanya lower (Unicode-aware)
    // let mut camel = String::new();
    // for w in words {
    //     let mut it = w.chars();
    //     if let Some(first) = it.next() {
    //         for up in first.to_uppercase() {
    //             camel.push(up);
    //         }
    //         for c in it.flat_map(|c| c.to_lowercase()) {
    //             camel.push(c);
    //         }
    //     }
    // }
    let mut camel = String::new();
    for w in words {
        let mut it = w.chars();
        if let Some(first) = it.next() {
            // Hanya naikkan huruf pertama kalau lowercase
            if first.is_lowercase() {
                for up in first.to_uppercase() {
                    camel.push(up);
                }
            } else {
                camel.push(first);
            }
            // Sisanya: biarkan seperti aslinya (tanpa force lowercase)
            camel.extend(it);
        }
    }

    // Jika diawali digit, prefix "_"
    if camel
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("_{}", camel)
    } else {
        camel
    }
}

pub fn generate_abbrev(name: &str) -> String {
    // Split words, filter out empty, normalize
    let words: Vec<&str> = name.split_whitespace().filter(|w| !w.is_empty()).collect();

    if words.is_empty() {
        return String::new();
    }

    let mut abbrev = String::new();

    // Case 1: If multiple words, take the first letter of first 3 words
    if words.len() > 1 {
        for w in words.iter().take(3) {
            if let Some(ch) = w.chars().next() {
                abbrev.push(ch.to_ascii_uppercase());
            }
        }
    } else {
        // Case 2: If only one word, take up to first 3 characters
        let word = words[0];
        abbrev = word
            .chars()
            .take(3)
            .collect::<String>()
            .to_ascii_uppercase();
    }

    abbrev
}

#[allow(dead_code)]
/// Trim leading and trailing quotes (single or double) from a string.
pub fn trim_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_quotes() {
        assert_eq!(trim_quotes(r#""hello""#), "hello");
        assert_eq!(trim_quotes(r#"'world'"#), "world");
        assert_eq!(trim_quotes(r#""mixed'"#), r#""mixed'"#);
        assert_eq!(trim_quotes("noquotes"), "noquotes");
        assert_eq!(trim_quotes("''"), "");
        assert_eq!(trim_quotes("\"\""), "");
        assert_eq!(trim_quotes("'"), "'");
        assert_eq!(trim_quotes("\""), "\"");
    }

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

    #[test]
    fn basic_spaces() {
        assert_eq!(to_pascalc("hello world"), "HelloWorld");
    }

    #[test]
    fn underscores_and_punct() {
        assert_eq!(to_pascalc("user_profile"), "UserProfile");
        assert_eq!(to_pascalc("user-profile  v2"), "UserProfileV2");
    }

    #[test]
    fn leading_digits_make_prefix() {
        assert_eq!(to_pascalc("123 cats"), "_123Cats");
        assert_eq!(to_pascalc("9_lives"), "_9Lives");
    }

    #[test]
    fn unicode_letters() {
        // Python 3 mengizinkan identifier Unicode; ini tetap dipertahankan.
        assert_eq!(to_pascalc("spécial chärs"), "SpécialChärs");
        assert_eq!(to_pascalc("日本 語_クラス"), "日本語クラス");
    }

    #[test]
    fn empty_or_symbols_only() {
        assert_eq!(to_pascalc(""), "_");
        assert_eq!(to_pascalc("?!@#$"), "_");
    }

    #[test]
    fn already_camelish() {
        assert_eq!(to_pascalc("AlreadyCamelCase"), "AlreadyCamelCase");
    }

    #[test]
    fn uppercase_handling() {
        assert_eq!(to_pascalc("SHU Policy"), "SHUPolicy");
        assert_eq!(to_pascalc("school project"), "SchoolProject");
        assert_eq!(to_pascalc("123abc"), "_123abc");
        assert_eq!(to_pascalc("hello_world"), "HelloWorld");
        assert_eq!(to_pascalc("alreadyCamel"), "AlreadyCamel");
    }

    #[test]
    fn test_abbrev() {
        assert_eq!(generate_abbrev("The Economist Magazine"), "TEM");
        assert_eq!(generate_abbrev("The Economist"), "TE");
        assert_eq!(generate_abbrev("Economist"), "ECO");
        assert_eq!(generate_abbrev("The Economist Magazine System"), "TEM");
        assert_eq!(generate_abbrev("A"), "A");
        assert_eq!(generate_abbrev(""), "");
    }
}
