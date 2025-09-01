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

pub fn run_bench_execute(
    config: &Config,
    _anal: &AnalyzedData,
    frappe_function: &str,
    args: Option<&str>,
    kwargs: Option<&str>,
) -> McpResult {
    let mut command_args = vec!["execute".to_string(), frappe_function.to_string()];
    
    if let Some(args_str) = args {
        command_args.push("--args".to_string());
        command_args.push(args_str.to_string());
    }
    
    if let Some(kwargs_str) = kwargs {
        command_args.push("--kwargs".to_string());
        command_args.push(kwargs_str.to_string());
    }
    
    let args_refs: Vec<&str> = command_args.iter().map(|s| s.as_str()).collect();
    
    shellutil::run_bench_command(config, &args_refs)
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, format!("{}", e), None))
        .and_then(|output| mcp_return!(output))
}