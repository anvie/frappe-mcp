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
    shellutil::run_bench_command(config, args)
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, format!("{}", e), None))
        .and_then(|output| mcp_return!(output))
}
