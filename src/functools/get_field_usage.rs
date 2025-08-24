#![allow(dead_code)]
use crate::analyze::AnalyzedData;
use crate::config::Config;
use rmcp::{model::*, ErrorData as McpError};
use serde_json::json;

type McpResult = Result<CallToolResult, McpError>;

pub fn get_field_usage(
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
            mcp_return!(serde_json::to_string_pretty(&json!({
                "error": "No symbol reference data available. Run analysis first.",
                "field_usage": []
            }))
            .unwrap());
        }
    };

    // Check if the doctype exists in symbol_refs
    let doctype_usage = match symbol_refs.doctypes.get(doctype) {
        Some(usage) => usage,
        None => {
            mcp_return!(serde_json::to_string_pretty(&json!({
                "doctype": doctype,
                "field_name": field_name,
                "message": format!("DocType '{}' not found in analyzed data", doctype),
                "field_usage": []
            }))
            .unwrap());
        }
    };

    // Check if the field exists for this doctype
    let field_occurrences = match doctype_usage.fields.get(field_name) {
        Some(occurrences) => occurrences,
        None => {
            mcp_return!(serde_json::to_string_pretty(&json!({
                "doctype": doctype,
                "field_name": field_name,
                "message": format!("Field '{}' not found for DocType '{}'", field_name, doctype),
                "field_usage": []
            }))
            .unwrap());
        }
    };

    // Limit the results
    let limited_occurrences: Vec<_> = field_occurrences.iter().take(limit).collect();

    let usage_data: Vec<serde_json::Value> = limited_occurrences
        .iter()
        .map(|occ| {
            json!({
                "file": occ.file,
                "line": occ.line,
                "variable": occ.var,
                "usage_type": occ.kind
            })
        })
        .collect();

    mcp_return!(serde_json::to_string(&json!({
        "doctype": doctype,
        "field_name": field_name,
        "total_occurrences": field_occurrences.len(),
        "showing": limited_occurrences.len(),
        "field_usage": usage_data
    }))
    .unwrap());
}

