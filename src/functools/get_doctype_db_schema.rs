#![allow(dead_code)]

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::shellutil;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

/// Run a bench command to get the database schema of a specified DocType
pub fn get_doctype_db_schema(config: &Config, _anal: &AnalyzedData, doctype: &str) -> McpResult {
    let args = ["mariadb", "-e", &format!("DESCRIBE `tab{}`\\G;", doctype)];
    shellutil::run_bench_command(config, args)
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, format!("{}", e), None))
        .and_then(|output| mcp_return!(output))
}
