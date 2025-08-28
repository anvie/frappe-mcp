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
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

#[derive(Debug, Clone)]
struct LinkInfo {
    pub target_doctype: String,
    pub field_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub link_type: LinkType,
}

#[derive(Debug, Clone)]
enum LinkType {
    Direct, // Direct Link field
    Table,  // Table field (child table)
    Select, // Select field with options referencing DocType
}

pub fn analyze_links(
    config: &Config,
    anal: &AnalyzedData,
    doctype: &str,
    depth: Option<usize>,
) -> McpResult {
    let max_depth = depth.unwrap_or(2);

    // Find the target DocType
    let target_doctype = anal
        .doctypes
        .iter()
        .find(|dt| dt.name.to_lowercase() == doctype.to_lowercase());

    let target_doctype = match target_doctype {
        Some(dt) => dt,
        None => {
            mcp_return!(format!("DocType '{}' not found in analyzed data", doctype));
        }
    };

    let mut result = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Start BFS from the target DocType
    queue.push_back((target_doctype.name.clone(), 0));

    while let Some((current_doctype, current_depth)) = queue.pop_front() {
        if visited.contains(&current_doctype) || current_depth > max_depth {
            continue;
        }

        visited.insert(current_doctype.clone());

        // Find links for current DocType
        let links = get_doctype_links(config, anal, &current_doctype)?;
        result.insert(current_doctype.clone(), links.clone());

        // Add connected DocTypes to queue for next level
        if current_depth < max_depth {
            for link in &links {
                if !visited.contains(&link.target_doctype) {
                    queue.push_back((link.target_doctype.clone(), current_depth + 1));
                }
            }
        }
    }

    // Format the results
    let formatted_result = format_link_analysis(doctype, &result, max_depth)?;
    mcp_return!(formatted_result)
}

fn get_doctype_links(
    config: &Config,
    anal: &AnalyzedData,
    doctype_name: &str,
) -> Result<Vec<LinkInfo>, McpError> {
    // Find DocType metadata
    let doctype_info = anal
        .doctypes
        .iter()
        .find(|dt| dt.name.to_lowercase() == doctype_name.to_lowercase());

    let doctype_info = match doctype_info {
        Some(info) => info,
        None => return Ok(Vec::new()),
    };

    // Read the JSON metadata file
    let meta_file = match &doctype_info.meta_file {
        Some(path) => path,
        None => return Ok(Vec::new()),
    };

    let meta_path = format!("{}/{}", config.app_absolute_path, meta_file);

    if !Path::new(&meta_path).exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&meta_path).map_err(|e| {
        McpError::new(
            ErrorCode::INVALID_REQUEST,
            format!("Failed to read metadata file: {}", e),
            Some(serde_json::json!({
                "file": meta_path
            })),
        )
    })?;

    let json: Value = serde_json::from_str(&content).map_err(|e| {
        McpError::new(
            ErrorCode::PARSE_ERROR,
            format!("Failed to parse JSON: {}", e),
            Some(serde_json::json!({
                "file": meta_path
            })),
        )
    })?;

    let mut links = Vec::new();

    // Extract fields array
    if let Some(fields) = json.get("fields").and_then(|f| f.as_array()) {
        for field in fields {
            if let Some(field_obj) = field.as_object() {
                if let Some(link_info) = extract_link_from_field(field_obj) {
                    links.push(link_info);
                }
            }
        }
    }

    Ok(links)
}

fn extract_link_from_field(field: &Map<String, Value>) -> Option<LinkInfo> {
    let fieldname = field.get("fieldname")?.as_str()?.to_string();
    let fieldtype = field.get("fieldtype")?.as_str()?;
    let label = field
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or(&fieldname);
    let reqd = field.get("reqd").and_then(|v| v.as_bool()).unwrap_or(false);

    match fieldtype {
        "Link" => {
            let options = field.get("options")?.as_str()?;
            Some(LinkInfo {
                target_doctype: options.to_string(),
                field_name: format!("{} ({})", label, fieldname),
                field_type: fieldtype.to_string(),
                is_required: reqd,
                link_type: LinkType::Direct,
            })
        }
        "Table" => {
            let options = field.get("options")?.as_str()?;
            Some(LinkInfo {
                target_doctype: options.to_string(),
                field_name: format!("{} ({})", label, fieldname),
                field_type: fieldtype.to_string(),
                is_required: reqd,
                link_type: LinkType::Table,
            })
        }
        "Select" => {
            let options = field.get("options").and_then(|v| v.as_str())?;

            // Check if options reference a DocType (simple heuristic)
            if options.contains('\n') {
                // Multi-line options, probably not a DocType reference
                return None;
            }

            // Check if it looks like a DocType name (contains spaces or is CamelCase)
            if options
                .chars()
                .any(|c| c.is_uppercase() && c.is_alphabetic())
                || options.contains(' ')
            {
                Some(LinkInfo {
                    target_doctype: options.to_string(),
                    field_name: format!("{} ({})", label, fieldname),
                    field_type: fieldtype.to_string(),
                    is_required: reqd,
                    link_type: LinkType::Select,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn format_link_analysis(
    root_doctype: &str,
    links_map: &HashMap<String, Vec<LinkInfo>>,
    max_depth: usize,
) -> Result<String, McpError> {
    let mut result = String::new();

    result.push_str(&format!(
        "🔗 Link Analysis for DocType: '{}'\n",
        root_doctype
    ));
    result.push_str(&format!("📊 Analysis Depth: {} levels\n", max_depth));
    result.push_str(&format!(
        "📈 Total DocTypes Analyzed: {}\n\n",
        links_map.len()
    ));

    // Group by link type for summary
    let mut total_direct = 0;
    let mut total_table = 0;
    let mut total_select = 0;

    for links in links_map.values() {
        for link in links {
            match link.link_type {
                LinkType::Direct => total_direct += 1,
                LinkType::Table => total_table += 1,
                LinkType::Select => total_select += 1,
            }
        }
    }

    result.push_str("📋 SUMMARY:\n");
    result.push_str(&format!("   • Direct Links: {}\n", total_direct));
    result.push_str(&format!("   • Child Tables: {}\n", total_table));
    result.push_str(&format!("   • Select References: {}\n\n", total_select));

    result.push_str("🌳 DETAILED ANALYSIS:\n");
    result.push_str("═".repeat(60).as_str());
    result.push('\n');

    // Show detailed breakdown for each DocType
    let mut sorted_doctypes: Vec<_> = links_map.keys().collect();
    sorted_doctypes.sort();

    for doctype_name in sorted_doctypes {
        let links = &links_map[doctype_name];

        result.push_str(&format!("\n📄 {}\n", doctype_name));
        result.push_str("─".repeat(40).as_str());
        result.push('\n');

        if links.is_empty() {
            result.push_str("   No outgoing links found.\n");
            continue;
        }

        // Group by link type
        let mut direct_links = Vec::new();
        let mut table_links = Vec::new();
        let mut select_links = Vec::new();

        for link in links {
            match link.link_type {
                LinkType::Direct => direct_links.push(link),
                LinkType::Table => table_links.push(link),
                LinkType::Select => select_links.push(link),
            }
        }

        if !direct_links.is_empty() {
            result.push_str("\n   🔗 Direct Links:\n");
            for link in direct_links {
                let req_marker = if link.is_required { "*" } else { "" };
                result.push_str(&format!(
                    "      → {} → {}{}\n",
                    link.field_name, link.target_doctype, req_marker
                ));
            }
        }

        if !table_links.is_empty() {
            result.push_str("\n   📋 Child Tables:\n");
            for link in table_links {
                let req_marker = if link.is_required { "*" } else { "" };
                result.push_str(&format!(
                    "      → {} → {}{}\n",
                    link.field_name, link.target_doctype, req_marker
                ));
            }
        }

        if !select_links.is_empty() {
            result.push_str("\n   📋 Select References:\n");
            for link in select_links {
                let req_marker = if link.is_required { "*" } else { "" };
                result.push_str(&format!(
                    "      → {} → {}{}\n",
                    link.field_name, link.target_doctype, req_marker
                ));
            }
        }
    }

    result.push_str("\n");
    result.push_str("═".repeat(60).as_str());
    result.push_str("\n📝 Legend: * = Required field\n");
    result.push_str("🔗 = Direct Link, 📋 = Child Table, 📋 = Select Reference\n");

    Ok(result)
}
