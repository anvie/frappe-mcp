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
use std::path::Path;
use std::process::Command;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn run_tests(
    config: &Config,
    anal: &AnalyzedData,
    module: Option<String>,
    doctype: Option<String>,
) -> McpResult {
    // let app_path = &config.app_absolute_path;

    // Verify we're in a Frappe bench directory
    let bench_path = find_bench_root(&config.frappe_bench_dir)?;

    let mut cmd_args: Vec<String> = vec![];

    let app_name_snake = to_snakec(&config.app_name);
    let snake_doctype = to_snakec(doctype.as_deref().unwrap_or(""));

    cmd_args.push("--site".to_string());
    cmd_args.push(config.site.clone());
    cmd_args.push("run-tests".to_string());

    // Build command arguments based on parameters
    match (module.as_ref(), doctype.as_ref()) {
        (Some(m), Some(_d)) => {
            // Test specific doctype in specific module
            let test_path = format!(
                "--app {} --module {}.{}.doctype.{}.test_{}",
                &app_name_snake,
                &app_name_snake,
                to_snakec(m),
                snake_doctype,
                snake_doctype
            );
            let test_args: Vec<String> = test_path
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            for arg in test_args {
                cmd_args.push(arg);
            }
        }
        (Some(m), None) => {
            // Test all doctypes in specific module
            let module_path = format!(
                "--app {} --module {}.{}",
                &app_name_snake,
                &app_name_snake,
                to_snakec(m),
            );
            let module_args: Vec<String> = module_path
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            for arg in module_args {
                cmd_args.push(arg);
            }
        }
        (None, Some(d)) => {
            // Find module for doctype and test it
            if let Some(found_module) = find_doctype_module(anal, d) {
                let snake_doctype = d.replace(' ', "_").to_lowercase();
                let test_path = format!(
                    "--app {} --module {}.{}.doctype.{}.test_{}",
                    &app_name_snake,
                    &app_name_snake,
                    to_snakec(&found_module),
                    snake_doctype,
                    snake_doctype
                );
                let test_args: Vec<String> = test_path
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                for arg in test_args {
                    cmd_args.push(arg);
                }
            } else {
                mcp_return!(format!("DocType '{}' not found in analyzed data", d));
            }
        }
        (None, None) => {
            // Test entire app
            cmd_args.push("--app".to_string());
            cmd_args.push(config.app_name.clone());
        }
    }

    // // Add test type specific flags
    // match test_type.as_str() {
    //     "unit" => {
    //         cmd_args.push("--skip-test-records".to_string());
    //     }
    //     "integration" => {
    //         cmd_args.push("--skip-before-setup".to_string());
    //     }
    //     "all" => {
    //         // Run all tests (default behavior)
    //     }
    //     _ => {
    //         mcp_return!(format!(
    //             "Invalid test_type '{}'. Valid options: unit, integration, all",
    //             test_type
    //         ));
    //     }
    // }

    if cmd_args.len() > 1 {
        tracing::debug!("Executing bench command: bench {}", cmd_args.join(" "));
    }

    // Execute bench command
    let output = Command::new("bench")
        .current_dir(&bench_path)
        .args(&cmd_args)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            let mut response = String::new();

            response.push_str("COMMAND EXECUTED:\n");
            response.push_str(&format!("bench {}\n\n", cmd_args.join(" ")));

            if !stdout.is_empty() {
                response.push_str("STDOUT:\n");
                response.push_str("─".repeat(50).as_str());
                response.push('\n');
                response.push_str(&stdout);
                response.push('\n');
                response.push_str("─".repeat(50).as_str());
                response.push_str("\n\n");
            }

            if !stderr.is_empty() {
                response.push_str("STDERR:\n");
                response.push_str("─".repeat(50).as_str());
                response.push('\n');
                response.push_str(&stderr);
                response.push('\n');
                response.push_str("─".repeat(50).as_str());
                response.push_str("\n\n");
            }

            // // Try to extract test summary
            // if let Some(summary) = extract_test_summary(&stdout) {
            //     response.push_str("TEST SUMMARY:\n");
            //     response.push_str(&summary);
            //     response.push('\n');
            // }

            response.push_str(&format!(
                "Exit code: {}\n",
                result.status.code().unwrap_or(-1)
            ));

            mcp_return!(response)
        }
        Err(e) => {
            mcp_return!(format!(
                "Failed to execute bench command: `bench {}`\n\n\
                Error: {}\n\n\
                \n\nMake sure:\n1. You're in a Frappe bench directory\n2. 'bench' command is available in PATH\n3. The app is installed in the bench",
                cmd_args.join(" "),
                e
            ));
        }
    }
}

fn find_bench_root(app_path: &str) -> Result<String, McpError> {
    let mut current = Path::new(app_path);

    if current.join("sites").exists() && current.join("apps").exists() {
        return Ok(current.to_string_lossy().to_string());
    }

    // Look for bench indicators going up the directory tree
    while let Some(parent) = current.parent() {
        let sites_dir = parent.join("sites");
        let apps_dir = parent.join("apps");

        if sites_dir.exists() && apps_dir.exists() {
            return Ok(parent.to_string_lossy().to_string());
        }

        current = parent;
    }

    Err(McpError::new(
        ErrorCode::INVALID_REQUEST,
        "Could not find Frappe bench root directory",
        Some(serde_json::json!({
            "searched_from": app_path
        })),
    ))
}

fn find_doctype_module(anal: &AnalyzedData, doctype_name: &str) -> Option<String> {
    anal.doctypes
        .iter()
        .find(|dt| dt.name.to_lowercase() == doctype_name.to_lowercase())
        .map(|dt| dt.module.clone())
}

// fn extract_test_summary(output: &str) -> Option<String> {
//     let lines: Vec<&str> = output.lines().collect();
//     let mut summary = Vec::new();
//     let mut in_summary = false;
//
//     for line in lines {
//         if line.contains("FAILED") || line.contains("PASSED") || line.contains("ERROR") {
//             in_summary = true;
//         }
//
//         if in_summary {
//             if line.contains("=")
//                 && (line.contains("passed") || line.contains("failed") || line.contains("error"))
//             {
//                 summary.push(line.to_string());
//                 break;
//             }
//
//             if line.contains("FAILED") || line.contains("ERROR") {
//                 summary.push(line.to_string());
//             }
//         }
//
//         // Look for coverage information
//         if line.contains("Total coverage:") || line.contains("TOTAL") {
//             summary.push(line.to_string());
//         }
//     }
//
//     if summary.is_empty() {
//         None
//     } else {
//         Some(summary.join("\n"))
//     }
// }
