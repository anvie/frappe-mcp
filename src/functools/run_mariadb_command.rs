#![allow(dead_code)]

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::shellutil;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn run_mariadb_command(config: &Config, _anal: &AnalyzedData, sql: &str) -> McpResult {
    shellutil::run_mariadb_command(config, sql)
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, format!("{}", e), None))
        .and_then(|output| mcp_return!(output))
}