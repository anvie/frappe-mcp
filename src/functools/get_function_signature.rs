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
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::fileutil::match_func_signature_in_file;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};
use serde_json::json;
use walkdir::WalkDir;

type McpResult = Result<CallToolResult, McpError>;

pub fn get_function_signature(
    config: &Config,
    anal: &AnalyzedData,
    name: &str,
    module: Option<String>,
    builtin: Option<bool>,
) -> McpResult {
    let module = module.unwrap_or("".to_string());
    let builtin = builtin.unwrap_or(false);

    let exts = vec!["py", "js"];

    let mut matches = Vec::new();

    if module != "" {
        let f_mod = anal
            .modules
            .iter()
            .find(|m| to_snakec(&m.name) == to_snakec(&module))
            .ok_or_else(|| {
                McpError::invalid_request("module_not_found", Some(json!({ "module": module })))
            })?;
        let candidate = format!("{}/{}", config.app_absolute_path, f_mod.location);
        tracing::info!("Searching in module path: {}", candidate);

        if Path::new(&candidate).exists() && Path::new(&candidate).is_dir() {
            for entry in WalkDir::new(&candidate).into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() {
                    continue;
                }
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if !exts.iter().any(|x| x == &ext) {
                        continue;
                    }
                } else {
                    continue;
                }
                if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                    continue;
                }
                if matches.len() > 2 {
                    break;
                }
            }
        } else {
            let out = format!(
                "Module path '{}' does not exist or is not a directory",
                candidate
            );
            mcp_return!(out);
        }
    }

    if builtin {
        for entry in WalkDir::new(&format!("{}/apps/frappe", config.frappe_bench_dir))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if !exts.iter().any(|x| x == &ext) {
                    continue;
                }
            } else {
                continue;
            }
            if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                continue;
            }
            if matches.len() > 2 {
                break;
            }
        }
    }

    if matches.is_empty() {
        for entry in WalkDir::new(&config.app_absolute_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if !exts.iter().any(|x| x == &ext) {
                    continue;
                }
            } else {
                continue;
            }

            if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                continue;
            }

            if matches.len() > 2 {
                break;
            }
        }
    }

    let out = if matches.is_empty() {
        format!(
            "No signature for '{}' found under '{}' (exts: {:?})",
            name, "??", exts
        )
    } else {
        format!(
            "Found signature(s) for '{}' in {} location(s):\n{}",
            name,
            matches.len(),
            matches
                .iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<String>>()
                .join("\n")
        )
    };

    mcp_return!(out)
}
