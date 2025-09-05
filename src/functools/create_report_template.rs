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

use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn create_report_template(
    config: &Config,
    _anal: &mut AnalyzedData,
    report_name: &str,
    module: &str,
    report_type: Option<String>,
    ref_doctype: Option<String>,
) -> McpResult {
    let snake_name = to_snakec(report_name);
    let snake_module = to_snakec(module);

    // Find the module directory
    let module_path = find_module_path(config, &snake_module)?;

    // Create report directory path
    let report_dir = format!("{}/report/{}", module_path, snake_name);

    // Create report directory if it doesn't exist
    if !Path::new(&report_dir).exists() {
        if let Err(e) = fs::create_dir_all(&report_dir) {
            mcp_return!(format!("Failed to create report directory: {}", e));
        }
    }

    let mut result = Vec::new();
    let report_type_str = report_type.unwrap_or_else(|| "Script Report".to_string());

    // 1. Create __init__.py
    let init_path = format!("{}/__init__.py", report_dir);
    if !Path::new(&init_path).exists() {
        if let Err(e) = fs::write(&init_path, "") {
            mcp_return!(format!("Failed to write __init__.py: {}", e));
        }
        result.push(format!("✓ Created __init__.py: {}", init_path));
    } else {
        tracing::info!("__init__.py already exists at: {}", init_path);
    }

    // 2. Create report Python file
    let py_content = generate_python_file(config, report_name, &snake_name, &ref_doctype);
    let py_path = format!("{}/{}.py", report_dir, snake_name);

    if !Path::new(&py_path).exists() {
        if let Err(e) = fs::write(&py_path, py_content) {
            mcp_return!(format!("Failed to write {}.py: {}", snake_name, e));
        }
        result.push(format!("✓ Created {}.py: {}", snake_name, py_path));
    } else {
        tracing::info!("{}.py already exists at: {}", snake_name, py_path);
    }

    // 3. Create report JavaScript file
    let js_content = generate_javascript_file(report_name, &ref_doctype);
    let js_path = format!("{}/{}.js", report_dir, snake_name);

    if !Path::new(&js_path).exists() {
        if let Err(e) = fs::write(&js_path, js_content) {
            mcp_return!(format!("Failed to write {}.js: {}", snake_name, e));
        }
        result.push(format!("✓ Created {}.js: {}", snake_name, js_path));
    } else {
        tracing::info!("{}.js already exists at: {}", snake_name, js_path);
    }

    // 4. Create report JSON metadata file (optional)
    let json_content = generate_json_file(report_name, module, &report_type_str, &ref_doctype);
    let json_path = format!("{}/{}.json", report_dir, snake_name);

    if !Path::new(&json_path).exists() {
        if let Err(e) = fs::write(&json_path, json_content) {
            mcp_return!(format!("Failed to write {}.json: {}", snake_name, e));
        }
        result.push(format!("✓ Created {}.json: {}", snake_name, json_path));
    } else {
        tracing::info!("{}.json already exists at: {}", snake_name, json_path);
    }

    let summary = format!(
        "Report template for '{}' created successfully in module '{}':\n\n{}\n\n\
        Next steps:\n\
        - Customize report logic in {}.py\n\
        - Configure filters in {}.js\n\
        - Test the report in Frappe: /app/query-report/{}",
        report_name,
        module,
        result.join("\n"),
        snake_name,
        snake_name,
        snake_name
    );

    mcp_return!(summary)
}

fn find_module_path(config: &Config, module: &str) -> Result<String, McpError> {
    let app_path = &config.app_absolute_path;
    let app_name = to_snakec(&config.app_name);

    // Search for the module in the app structure
    let search_pattern = format!("{}/{}/{}", app_path, app_name, module);

    if Path::new(&search_pattern).exists() && Path::new(&search_pattern).is_dir() {
        return Ok(search_pattern);
    }

    Err(McpError {
        code: rmcp::model::ErrorCode(-1),
        message: format!(
            "Module '{}' not found in app structure. Available path should be: {}",
            module, search_pattern
        )
        .into(),
        data: None,
    })
}

fn generate_python_file(
    config: &Config,
    _report_name: &str,
    _snake_name: &str,
    ref_doctype: &Option<String>,
) -> String {
    let current_year = Utc::now().format("%Y");
    let app_snake = to_snakec(&config.app_name);

    let ref_import = if let Some(ref_dt) = ref_doctype {
        format!("\n# Uncomment if you need to import the reference DocType\n# from {}.{}.doctype.{}.{} import {}", 
                app_snake, app_snake, to_snakec(ref_dt), to_snakec(ref_dt), to_snakec(ref_dt))
    } else {
        "".to_string()
    };

    format!(
        r#"# Copyright (c) {}, {}
# For license information, please see license.txt

from __future__ import unicode_literals
import frappe
from frappe import _{}

def execute(filters=None):
    """
    Main report execution function
    Returns: columns, data
    """
    columns, data = [], []
    
    columns = get_columns()
    data = get_data(filters)
    
    return columns, data

def get_columns():
    """
    Define report columns
    """
    return [
        {{
            "fieldname": "name",
            "label": _("Name"),
            "fieldtype": "Data",
            "width": 200
        }},
        {{
            "fieldname": "creation",
            "label": _("Creation Date"),
            "fieldtype": "Date",
            "width": 120
        }}
        # TODO: Add more columns based on your requirements
    ]

def get_data(filters):
    """
    Fetch and process report data
    """
    # TODO: Implement your report data logic here
    
    conditions = get_conditions(filters)
    
    # Example query - replace with your actual logic
    data = frappe.db.sql("""
        SELECT 
            name,
            creation
        FROM `tabYour DocType`
        WHERE 1=1 {{conditions}}
        ORDER BY creation DESC
    """.format(conditions=conditions), as_dict=1)
    
    return data

def get_conditions(filters):
    """
    Build WHERE conditions based on filters
    """
    conditions = ""
    
    if filters.get("company"):
        conditions += " AND company = %(company)s"
    
    if filters.get("from_date"):
        conditions += " AND creation >= %(from_date)s"
    
    if filters.get("to_date"):
        conditions += " AND creation <= %(to_date)s"
    
    # TODO: Add more filter conditions
    
    return conditions
"#,
        current_year, config.app_name, ref_import
    )
}

fn generate_javascript_file(report_name: &str, ref_doctype: &Option<String>) -> String {
    let company_filter = r#"        {
            fieldname: "company",
            label: __("Company"),
            fieldtype: "Link",
            options: "Company",
            default: frappe.defaults.get_user_default("Company")
        },"#;

    let ref_filter = if let Some(ref_dt) = ref_doctype {
        format!(
            r#"        {{
            fieldname: "{}",
            label: __("{}"),
            fieldtype: "Link",
            options: "{}"
        }},"#,
            to_snakec(ref_dt),
            ref_dt,
            ref_dt
        )
    } else {
        "".to_string()
    };

    format!(
        r#"frappe.query_reports["{}"] = {{
    filters: [
{}{}
        {{
            fieldname: "from_date",
            label: __("From Date"),
            fieldtype: "Date",
            default: frappe.datetime.add_months(frappe.datetime.get_today(), -1)
        }},
        {{
            fieldname: "to_date", 
            label: __("To Date"),
            fieldtype: "Date",
            default: frappe.datetime.get_today()
        }}
        // TODO: Add more filters as needed
    ]
}};
"#,
        report_name,
        company_filter,
        if !ref_filter.is_empty() {
            format!("\n{}", ref_filter)
        } else {
            "".to_string()
        }
    )
}

fn generate_json_file(
    report_name: &str,
    module: &str,
    report_type: &str,
    ref_doctype: &Option<String>,
) -> String {
    let current_time = Utc::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
    let ref_doctype_str = ref_doctype.as_deref().unwrap_or("");

    let json_content = serde_json::json!({
        "add_total_row": 0,
        "creation": current_time,
        "disable_prepared_report": 0,
        "disabled": 0,
        "docstatus": 0,
        "doctype": "Report",
        "idx": 0,
        "is_standard": "Yes",
        "module": module,
        "name": report_name,
        "owner": "Administrator",
        "prepared_report": 0,
        "ref_doctype": ref_doctype_str,
        "report_name": report_name,
        "report_type": report_type,
        "roles": [
            {
                "role": "System Manager"
            }
        ]
    });

    serde_json::to_string_pretty(&json_content)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to generate JSON: {}\"}}", e))
}
