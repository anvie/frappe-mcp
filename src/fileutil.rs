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
