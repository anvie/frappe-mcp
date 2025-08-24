#![allow(dead_code)]
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};
use walkdir::WalkDir;

type McpResult = Result<CallToolResult, McpError>;

pub fn get_doctype(config: &Config, anal: &AnalyzedData, name: &str, json_only: bool) -> McpResult {
    let target = name;
    let mut hits: Vec<String> = Vec::new();

    let candidate = anal
        .doctypes
        .iter()
        .find(|a| a.name.to_lowercase() == target.to_lowercase());

    if let Some(doc) = candidate {
        if json_only {
            if doc.meta_file.is_none() {
                mcp_return!(format!(
                    "DocType '{}' found, but has no metadata file",
                    doc.name
                ));
            } else {
                // read whole metadata file
                let meta_path = format!(
                    "{}/{}",
                    config.app_absolute_path,
                    doc.meta_file.as_ref().unwrap()
                );
                if !Path::new(&meta_path).exists() {
                    mcp_return!(format!(
                        "DocType '{}' metadata file '{}' not found",
                        target, meta_path
                    ));
                }
                let content = std::fs::read_to_string(meta_path).unwrap_or_else(|_| "".to_string());
                mcp_return!(format!("{}", content));
            }
        }

        let mut msg = format!("DocType '{}' found:\n\n", doc.name);
        msg.push_str(&format!("- Module: {}\n", doc.module));
        msg.push_str(&format!("- Backend: {}\n", doc.backend_file));
        if let Some(front) = &doc.frontend_file {
            msg.push_str(&format!("- Frontend: {}\n", front));
        }
        if let Some(meta_file) = &doc.meta_file {
            msg.push_str(&format!("- Metadata: {}\n", meta_file));
        }
        mcp_return!(msg);
    }

    let target_pyname = to_snakec(&target);
    let root = &config.app_absolute_path;
    let candidate = format!("{}/{}", root, target_pyname);

    // direct relative candidate
    if !Path::new(&candidate).exists() {
        mcp_return!(format!("DocType '{}' not found under '{}'", target, root));
    }

    // full-tree search for both file name and in-file markers
    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let path_str = p.display().to_string();

        if path_str.ends_with(&format!("{}/{}.json", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            hits.push(format!("- Metadata: {}", path_str));
            continue;
        }

        if path_str.ends_with(&format!("{}/{}.js", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            hits.push(format!("- Frontend: {}", path_str));
            continue;
        }

        if path_str.ends_with(&format!("{}/{}.py", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            hits.push(format!("- Backend: {}", path_str));
            continue;
        }
    }

    let out = if hits.is_empty() {
        format!("DocType '{}' not found under '{}'", target, root)
    } else {
        format!("DocType '{}' found:\n{}", target, hits.join("\n"))
    };

    mcp_return!(out)
}
