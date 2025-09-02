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
use crate::analyze::AnalyzedData;
use crate::config::Config;
use rmcp::{model::*, ErrorData as McpError};
use std::fs;
use std::io::{BufRead, BufReader};

type McpResult = Result<CallToolResult, McpError>;

fn read_code_snippet(
    file_path: &str,
    target_line: usize,
    context_lines: usize,
) -> Option<Vec<(usize, String)>> {
    let file = fs::File::open(file_path).ok()?;
    let reader = BufReader::new(file);
    let mut lines: Vec<(usize, String)> = Vec::new();

    let start_line = target_line.saturating_sub(context_lines);
    let end_line = target_line + context_lines;

    for (idx, line_result) in reader.lines().enumerate() {
        let line_number = idx + 1;

        if line_number >= start_line && line_number <= end_line {
            if let Ok(line) = line_result {
                lines.push((line_number, line));
            }
        }

        if line_number > end_line {
            break;
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

pub fn find_field_usage(
    _config: &Config,
    anal: &AnalyzedData,
    doctype: &str,
    field_name: &str,
    limit: Option<usize>,
) -> McpResult {
    let limit = limit.unwrap_or(10);

    // Check if symbol_refs data is available
    let symbol_refs = match &anal.symbol_refs {
        Some(refs) => refs,
        None => {
            mcp_return!("No symbol reference data available. Run analysis first.");
        }
    };

    // Check if the doctype exists in symbol_refs
    let doctype_usage = match symbol_refs.doctypes.get(doctype) {
        Some(usage) => usage,
        None => {
            mcp_return!(format!("DocType '{}' not found in analyzed data", doctype));
        }
    };

    // Check if the field exists for this doctype
    let field_occurrences = match doctype_usage.fields.get(field_name) {
        Some(occurrences) => occurrences,
        None => {
            mcp_return!(format!(
                "Field '{}' not found for DocType '{}'",
                field_name, doctype
            ));
        }
    };

    // Limit the results
    let limited_occurrences: Vec<_> = field_occurrences.iter().take(limit).collect();

    // Prepare the result in human friendly format
    let mut result = vec![];
    result.push(format!(
        "Found {} occurrences of field usage `{}` of doctype `{}`:",
        field_occurrences.len(),
        field_name,
        doctype,
    ));

    for (idx, occ) in limited_occurrences.iter().enumerate() {
        result.push(String::new());
        result.push(format!(
            "{}. In file '{}' at line {}:",
            idx + 1,
            occ.file,
            occ.line
        ));

        // Try to read the code snippet
        if let Some(snippet_lines) = read_code_snippet(&occ.file, occ.line, 2) {
            // Find the maximum line number width for proper alignment
            let max_line_width = snippet_lines
                .iter()
                .map(|(line_no, _)| line_no.to_string().len())
                .max()
                .unwrap_or(1);

            for (line_no, content) in &snippet_lines {
                let is_target_line = *line_no == occ.line;
                let arrow = if is_target_line { "→" } else { " " };

                result.push(format!(
                    "   {:>width$}: {} {}",
                    line_no,
                    arrow,
                    content,
                    width = max_line_width
                ));
            }
        } else {
            result.push(format!("   [Could not read file content]"));
        }
    }

    if field_occurrences.len() > limit {
        result.push(String::new());
        result.push(format!(
            "... and {} more occurrences (showing first {} only)",
            field_occurrences.len() - limit,
            limit
        ));
    }

    mcp_return!(result.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_code_snippet_middle_of_file() {
        let test_file = "test_data/sample_code.py";
        let result = read_code_snippet(test_file, 9, 2);

        assert!(result.is_some());
        let lines = result.unwrap();

        // println!(
        //     "Lines: {}",
        //     lines
        //         .iter()
        //         .map(|(n, l)| format!("{}: {}", n, l))
        //         .collect::<Vec<String>>()
        //         .join("\n")
        );

        // Should have 5 lines total (target line 9 + 2 before + 2 after)
        assert_eq!(lines.len(), 5);

        // Check line numbers
        assert_eq!(lines[0].0, 7);
        assert_eq!(lines[1].0, 8);
        assert_eq!(lines[2].0, 9); // Target line
        assert_eq!(lines[3].0, 10);
        assert_eq!(lines[4].0, 11);

        // Check content
        assert!(lines[2].1.contains("if task.status == \"Open\":"));
    }

    #[test]
    fn test_read_code_snippet_beginning_of_file() {
        let test_file = "test_data/sample_code.py";
        let result = read_code_snippet(test_file, 2, 2);

        assert!(result.is_some());
        let lines = result.unwrap();

        // Should have 4 lines (no line 0, starts at line 1)
        assert_eq!(lines.len(), 4);

        assert_eq!(lines[0].0, 1);
        assert_eq!(lines[1].0, 2); // Target line
        assert_eq!(lines[2].0, 3);
        assert_eq!(lines[3].0, 4);

        assert!(lines[1].1.contains("# Line 2"));
    }

    #[test]
    fn test_read_code_snippet_end_of_file() {
        let test_file = "test_data/sample_code.py";
        let result = read_code_snippet(test_file, 51, 2);

        assert!(result.is_some());
        let lines = result.unwrap();

        // File has 52 lines, so line 51 with 2 context would be:
        // lines 49, 50, 51, 52 (and potentially 53 if it existed)
        // But since file ends at 52, we get 49-52
        assert!(lines.len() >= 4);

        // Find the line with number 51
        let target_line = lines.iter().find(|(num, _)| *num == 51);
        assert!(target_line.is_some());
        assert!(target_line.unwrap().1.contains("return Result(True)"));
    }

    #[test]
    fn test_read_code_snippet_nonexistent_file() {
        let test_file = "test_data/nonexistent.py";
        let result = read_code_snippet(test_file, 10, 2);

        assert!(result.is_none());
    }

    #[test]
    fn test_read_code_snippet_beyond_file_end() {
        let test_file = "test_data/sample_code.py";
        let result = read_code_snippet(test_file, 100, 2);

        // When requesting a line beyond the file, we should get None or empty result
        if let Some(lines) = result {
            assert!(lines.is_empty());
        }
    }

    #[test]
    fn test_read_code_snippet_single_context_line() {
        let test_file = "test_data/sample_code.py";
        let result = read_code_snippet(test_file, 20, 1);

        assert!(result.is_some());
        let lines = result.unwrap();

        // Should have 3 lines (19, 20, 21)
        assert_eq!(lines.len(), 3);

        assert_eq!(lines[0].0, 19);
        assert_eq!(lines[1].0, 20); // Target line
        assert_eq!(lines[2].0, 21);

        assert!(lines[1].1.contains("# Line 20"));
    }
}
