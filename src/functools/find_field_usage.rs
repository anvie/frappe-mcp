#![allow(dead_code)]
use crate::analyze::AnalyzedData;
use crate::config::Config;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn find_field_usage(
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
            mcp_return!("No symbol reference data available. Run analysis first.");
        }
    };

    // Check if the doctype exists in symbol_refs
    let doctype_usage = match symbol_refs.doctypes.get(doctype) {
        Some(usage) => usage,
        None => {
            mcp_return!(format!("DocType '{}' not found in analyzed data", doctype));
        }
    };

    // Check if the field exists for this doctype
    let field_occurrences = match doctype_usage.fields.get(field_name) {
        Some(occurrences) => occurrences,
        None => {
            mcp_return!(format!(
                "Field '{}' not found for DocType '{}'",
                field_name, doctype
            ));
        }
    };

    // Limit the results
    let limited_occurrences: Vec<_> = field_occurrences.iter().take(limit).collect();

    // Prepare the result in human friendly format
    let mut result = vec![];
    result.push(format!(
        "Found {} occurrences of field usage `{}` of doctype `{}`:",
        field_occurrences.len(),
        field_name,
        doctype,
    ));

    for occ in limited_occurrences {
        result.push(format!("- In file '{}' at line {}", occ.file, occ.line,));
    }

    mcp_return!(result.join("\n"))
}
