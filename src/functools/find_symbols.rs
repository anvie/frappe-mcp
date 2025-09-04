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
use regex::Regex;
use rmcp::{model::*, ErrorData as McpError};
use std::fs;
use std::io::{BufRead, BufReader};
use walkdir::WalkDir;

type McpResult = Result<CallToolResult, McpError>;

#[derive(Debug, Clone)]
struct ScoredMatch {
    path: String,
    line_no: usize,
    content: String,
    score: f64,
}

fn calculate_fuzzy_score(pattern: &str, text: &str) -> f64 {
    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();

    // Exact match gets highest score
    if text_lower.contains(&pattern_lower) {
        return 100.0;
    }

    // Calculate character match ratio
    let pattern_chars: Vec<char> = pattern_lower.chars().collect();
    let text_chars: Vec<char> = text_lower.chars().collect();

    let mut matched_chars = 0;
    let mut pattern_idx = 0;
    let mut consecutive_matches = 0;
    let mut max_consecutive = 0;

    for &text_char in &text_chars {
        if pattern_idx < pattern_chars.len() && text_char == pattern_chars[pattern_idx] {
            matched_chars += 1;
            pattern_idx += 1;
            consecutive_matches += 1;
            max_consecutive = max_consecutive.max(consecutive_matches);
        } else {
            consecutive_matches = 0;
        }
    }

    if matched_chars == 0 {
        return 0.0;
    }

    // Score based on:
    // 1. Character match ratio (0-50 points)
    // 2. Pattern completion ratio (0-30 points)
    // 3. Consecutive match bonus (0-20 points)
    let char_ratio = (matched_chars as f64 / pattern_chars.len() as f64) * 50.0;
    let completion_ratio = (pattern_idx as f64 / pattern_chars.len() as f64) * 30.0;
    let consecutive_bonus = (max_consecutive as f64 / pattern_chars.len() as f64) * 20.0;

    char_ratio + completion_ratio + consecutive_bonus
}

pub fn find_symbols(
    config: &Config,
    _anal: &AnalyzedData,
    name: &str,
    search_in: Option<String>,
    fuzzy: Option<bool>,
    limit: Option<usize>,
) -> McpResult {
    let search_in = search_in.unwrap_or_else(|| "all".to_string());
    let fuzzy = fuzzy.unwrap_or(false);
    let limit = limit.unwrap_or(50);

    // Set file extensions based on search type
    let exts = match search_in.as_str() {
        "backend" => vec!["py"],
        "frontend" => vec!["js", "ts", "html", "css"],
        _ => vec!["py", "js", "css", "ts", "json", "html"], // "all" or any other value
    };

    let mut scored_matches = Vec::new();

    // For fuzzy matching, we'll score all potential matches
    // For exact matching, use regex as before
    let re = if !fuzzy {
        Some(
            Regex::new(&format!(r"(?i)\b{}\b", regex::escape(name))).map_err(|e| {
                McpError::invalid_request(
                    "invalid_regex",
                    Some(serde_json::json!({ "error": e.to_string() })),
                )
            })?,
        )
    } else {
        None
    };

    // Search in the app directory
    for entry in WalkDir::new(&config.app_absolute_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        // Check if file has one of the allowed extensions
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            if !exts.iter().any(|x| x == &ext) {
                continue;
            }
        } else {
            continue;
        }

        // Skip hidden files and directories
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        {
            continue;
        }

        // Skip common non-source directories
        let path_str = entry.path().display().to_string();
        if path_str.contains("/__pycache__/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.git/")
            || path_str.contains("/build/")
            || path_str.contains("/dist/")
        {
            continue;
        }

        // Read file content and search for the symbol
        if let Ok(content) = fs::read_to_string(entry.path()) {
            // Precompute line starts for line number calculation
            let mut line_starts = Vec::with_capacity(256);
            line_starts.push(0);
            for (i, b) in content.bytes().enumerate() {
                if b == b'\n' {
                    line_starts.push(i + 1);
                }
            }

            let byte_to_line_number = |offset: usize| -> usize {
                match line_starts.binary_search(&offset) {
                    Ok(i) => i + 1,
                    Err(i) => i,
                }
            };

            // Get relative path from the app directory
            let relative_path = entry
                .path()
                .strip_prefix(&config.app_absolute_path)
                .unwrap_or(entry.path())
                .display()
                .to_string();

            if fuzzy {
                // For fuzzy matching, check each line for potential matches
                for (line_idx, line) in content.lines().enumerate() {
                    let score = calculate_fuzzy_score(name, line);
                    if score > 20.0 {
                        // Only include matches above threshold
                        scored_matches.push(ScoredMatch {
                            path: relative_path.clone(),
                            line_no: line_idx + 1,
                            content: line.trim().to_string(),
                            score,
                        });
                    }
                }
            } else if let Some(ref regex) = re {
                // For exact matching, use regex as before
                for mat in regex.find_iter(&content) {
                    let line_no = byte_to_line_number(mat.start());
                    let start_line_idx = line_starts
                        .get(line_no.saturating_sub(1))
                        .copied()
                        .unwrap_or(0);
                    let end_line_idx = line_starts.get(line_no).copied().unwrap_or(content.len());

                    // Extract the line containing the match
                    let line_content = content[start_line_idx..end_line_idx]
                        .trim_end_matches('\n')
                        .trim_end_matches('\r');

                    scored_matches.push(ScoredMatch {
                        path: relative_path.clone(),
                        line_no,
                        content: line_content.trim().to_string(),
                        score: 100.0, // Exact matches get max score
                    });
                }
            }
        }
    }

    // Sort matches by score (highest first) and take the limit
    scored_matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_matches: Vec<_> = scored_matches.into_iter().take(limit).collect();

    let out = if top_matches.is_empty() {
        format!(
            "No symbols matching '{}' found in {} (search: {}, fuzzy: {})",
            name, search_in, search_in, fuzzy
        )
    } else {
        let display_count = top_matches.len();
        let header = format!("Found {} symbols matching '{}':\n", display_count, name);

        let mut matches_str = Vec::new();
        for (idx, m) in top_matches.iter().enumerate() {
            matches_str.push(String::new());

            if fuzzy {
                matches_str.push(format!(
                    "{}. In file '{}' at line {} (score: {:.1}):",
                    idx + 1,
                    m.path,
                    m.line_no,
                    m.score
                ));
            } else {
                matches_str.push(format!(
                    "{}. In file '{}' at line {}:",
                    idx + 1,
                    m.path,
                    m.line_no
                ));
            }

            // Try to read the code snippet
            let full_path = format!("{}/{}", config.app_absolute_path, m.path);
            if let Some(snippet_lines) = read_code_snippet(&full_path, m.line_no, 2) {
                // Find the maximum line number width for proper alignment
                let max_line_width = snippet_lines
                    .iter()
                    .map(|(line_no, _)| line_no.to_string().len())
                    .max()
                    .unwrap_or(1);

                for (line_no, content) in &snippet_lines {
                    let is_target_line = *line_no == m.line_no;
                    let arrow = if is_target_line { "→" } else { " " };

                    matches_str.push(format!(
                        "   {:>width$}: {} {}",
                        line_no,
                        arrow,
                        content,
                        width = max_line_width
                    ));
                }
            } else {
                matches_str.push(format!("   [Could not read file content]"));
            }
        }

        let matches_string = matches_str.join("\n");
        format!("{}{}", header, matches_string)
    };

    mcp_return!(out)
}

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
