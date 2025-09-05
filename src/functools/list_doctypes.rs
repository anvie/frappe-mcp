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
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn list_doctypes(
    _config: &Config,
    anal: &AnalyzedData,
    module_filter: Option<String>,
) -> McpResult {
    let mut result: Vec<String> = Vec::new();

    // Filter doctypes by module if specified
    let (doctypes, filtered_module_name) = if let Some(ref module) = module_filter {
        let filtered = anal
            .doctypes
            .iter()
            .filter(|dt| dt.module.to_lowercase() == module.to_lowercase())
            .collect::<Vec<_>>();
        (filtered, Some(module.clone()))
    } else {
        (anal.doctypes.iter().collect::<Vec<_>>(), None)
    };

    if doctypes.is_empty() {
        let msg = if let Some(module_name) = filtered_module_name {
            format!("No DocTypes found in module '{}'", module_name)
        } else {
            "No DocTypes found in the current app".to_string()
        };
        mcp_return!(msg);
    }

    let doctype_count = doctypes.len();

    // Group by module for better organization
    let mut modules = std::collections::HashMap::new();
    for doctype in &doctypes {
        modules
            .entry(&doctype.module)
            .or_insert_with(Vec::new)
            .push(doctype);
    }

    // Sort modules by name
    let mut module_names: Vec<_> = modules.keys().collect();
    module_names.sort();

    let total_count = if filtered_module_name.is_some() {
        doctype_count
    } else {
        anal.doctypes.len()
    };

    result.push(format!(
        "Found {} DocType(s) across {} module(s):\n",
        total_count,
        modules.len()
    ));

    for module_name in module_names {
        let mut module_doctypes = modules[module_name].clone();
        module_doctypes.sort_by(|a, b| a.name.cmp(&b.name));

        result.push(format!("## Module: {}", module_name));
        result.push(format!(
            "   ({} DocType{})",
            module_doctypes.len(),
            if module_doctypes.len() == 1 { "" } else { "s" }
        ));

        for doctype in module_doctypes {
            result.push(format!("   - {}", doctype.name));
        }
        result.push("".to_string()); // Empty line between modules
    }

    mcp_return!(result.join("\n"))
}
