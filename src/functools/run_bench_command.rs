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
use crate::shellutil;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn run_bench_command(config: &Config, _anal: &AnalyzedData, args: &[&str]) -> McpResult {
    // if migrate is in args, then remove the lock file, sometimes migrate fails because of the
    // lock file in dev environment.
    if args.contains(&"migrate") {
        tracing::trace!("Removing lock files before migrate");
        let lock_file_path = format!("{}/sites/{}/locks/*", config.frappe_bench_dir, config.site);
        tracing::trace!("Lock file path: {}", lock_file_path);
        // delete all files inside the lock_file_path
        // iterate over all files in the lock_file_path and delete them
        for entry in glob::glob(&lock_file_path).unwrap() {
            match entry {
                Ok(path) => {
                    tracing::trace!("Removing lock file: {:?}", path);
                    if std::fs::remove_file(&path).is_err() {
                        tracing::warn!("Failed to remove lock file: {:?}", path);
                    }
                }
                Err(_) => {
                    // ignore error
                }
            }
        }
    }
    shellutil::run_bench_command(config, args)
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, format!("{}", e), None))
        .and_then(|output| mcp_return!(output))
}
