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
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::{generate_abbrev, to_pascalc, to_snakec};
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn create_test_template(
    config: &Config,
    _anal: &mut AnalyzedData,
    doctype: &str,
    doctype_dependencies: Option<Vec<String>>,
) -> McpResult {
    let snake_name = to_snakec(doctype);

    // Find the DocType directory by searching for the JSON metadata file
    let doctype_path = find_doctype_path(config, doctype)?;

    let mut result = Vec::new();
    let dependencies = doctype_dependencies.unwrap_or_default();

    // 1. Create test_records.json
    let test_records_content = generate_test_records_json(config, doctype, &doctype_path)?;
    let test_records_path = format!("{}/test_records.json", doctype_path);

    if Path::new(&test_records_path).exists() {
        mcp_return!(format!(
            "test_records.json already exists at: {}",
            test_records_path
        ));
    }

    if let Err(e) = fs::write(&test_records_path, test_records_content) {
        mcp_return!(format!("Failed to write test_records.json: {}", e));
    }
    result.push(format!(
        "✓ Created test_records.json: {}",
        test_records_path
    ));

    // 2. Create test_[doctype_name].py
    let test_py_content = generate_test_python_file(config, doctype, &snake_name, &dependencies);
    let test_py_path = format!("{}/test_{}.py", doctype_path, snake_name);

    if Path::new(&test_py_path).exists() {
        // mcp_return!(format!(
        //     "test_{}.py already exists at: {}",
        //     snake_name, test_py_path
        // ));
        tracing::info!("test_{}.py already exists at: {}", snake_name, test_py_path)
    } else {
        if let Err(e) = fs::write(&test_py_path, test_py_content) {
            mcp_return!(format!("Failed to write test_{}.py: {}", snake_name, e));
        }
        result.push(format!(
            "✓ Created test_{}.py: {}",
            snake_name, test_py_path
        ));
    }

    let summary = format!(
        "Test template for '{}' created successfully:\n\n{}\n\nNext steps:\n- Run tests using: bench run-tests --doctype \"{}\"\n- Customize test data in test_records.json\n- Add test methods in test_{}.py",
        doctype,
        result.join("\n"),
        doctype,
        snake_name
    );

    mcp_return!(summary)
}

fn find_doctype_path(config: &Config, doctype: &str) -> Result<String, McpError> {
    let snake_name = to_snakec(doctype);
    let app_path = &config.app_absolute_path;
    let app_name = to_snakec(&config.app_name);

    // Search for the DocType in common module locations
    let search_paths = vec![format!(
        "{}/{}/*/doctype/{}",
        app_path, app_name, snake_name
    )];

    for search_pattern in search_paths {
        if let Ok(entries) = glob::glob(&search_pattern) {
            for path in entries.flatten() {
                if path.is_dir() {
                    let json_file = format!("{}/{}.json", path.display(), snake_name);
                    if Path::new(&json_file).exists() {
                        return Ok(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Err(McpError {
        code: rmcp::model::ErrorCode(-1),
        message: format!(
            "DocType '{}' not found in app structure. Make sure the DocType exists first.",
            doctype
        )
        .into(),
        data: None,
    })
}

fn generate_test_records_json(
    _config: &Config,
    doctype: &str,
    doctype_path: &str,
) -> Result<String, McpError> {
    let snake_name = to_snakec(doctype);
    let json_metadata_path = format!("{}/{}.json", doctype_path, snake_name);

    // Read the DocType JSON metadata to extract fields
    let metadata_content = fs::read_to_string(&json_metadata_path).map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-1),
        message: format!("Failed to read DocType metadata: {}", e).into(),
        data: None,
    })?;

    let metadata: Value = serde_json::from_str(&metadata_content).map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-1),
        message: format!("Failed to parse DocType metadata JSON: {}", e).into(),
        data: None,
    })?;

    let fields = metadata["fields"].as_array().ok_or_else(|| McpError {
        code: rmcp::model::ErrorCode(-1),
        message: "No fields found in DocType metadata".to_string().into(),
        data: None,
    })?;

    // Generate sample test record
    let mut test_record = serde_json::json!({
        "doctype": doctype
    });

    // Add naming series if present
    if let Some(naming_series_field) = fields
        .iter()
        .find(|f| f["fieldtype"] == "Select" && f["fieldname"] == "naming_series")
    {
        if let Some(options) = naming_series_field["options"].as_str() {
            let default_series = format!("{}-.#####", generate_abbrev(doctype));
            let first_option = options.lines().next().unwrap_or(&default_series);
            test_record["naming_series"] = serde_json::Value::String(first_option.to_string());
        }
    }

    // Generate sample data for each field
    for field in fields {
        let fieldname = field["fieldname"].as_str().unwrap_or("");
        let fieldtype = field["fieldtype"].as_str().unwrap_or("");
        let label = field["label"].as_str().unwrap_or("");

        // Skip standard fields and section breaks
        if [
            "naming_series",
            "name",
            "creation",
            "modified",
            "modified_by",
            "owner",
            "docstatus",
            "idx",
        ]
        .contains(&fieldname)
            || fieldtype == "Section Break"
            || fieldtype == "Column Break"
        {
            continue;
        }

        let sample_value = generate_sample_field_value(fieldtype, label, fieldname);
        if let Some(value) = sample_value {
            test_record[fieldname] = value;
        }
    }

    let test_records = vec![test_record];

    serde_json::to_string_pretty(&test_records).map_err(|e| McpError {
        code: rmcp::model::ErrorCode(-1),
        message: format!("Failed to serialize test records JSON: {}", e).into(),
        data: None,
    })
}

fn generate_sample_field_value(fieldtype: &str, label: &str, fieldname: &str) -> Option<Value> {
    match fieldtype {
        "Data" | "Small Text" => Some(Value::String(format!("_Test {}", label))),
        "Text" | "Text Editor" => Some(Value::String(format!("_Test {} content", label))),
        "Int" => Some(Value::Number(serde_json::Number::from(1))),
        "Float" | "Currency" | "Percent" => {
            Some(Value::Number(serde_json::Number::from_f64(100.0).unwrap()))
        }
        "Check" => Some(Value::Bool(false)),
        "Date" => {
            let today = Utc::now().format("%Y-%m-%d").to_string();
            Some(Value::String(today))
        }
        "Datetime" => {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            Some(Value::String(now))
        }
        "Time" => Some(Value::String("09:00:00".to_string())),
        "Link" => {
            // For link fields, we'll use a generic test value
            // Users should customize this based on their linked DocTypes
            Some(Value::String(format!("_Test {}", label)))
        }
        "Select" => {
            // For select fields, we'll use the first option or a default
            Some(Value::String(format!("_Test {}", label)))
        }
        "Table" => {
            // For child table fields, create an empty array
            // Users should add child records as needed
            Some(Value::Array(vec![]))
        }
        "Attach" | "Attach Image" => Some(Value::String("/files/test_file.txt".to_string())),
        _ => {
            // For unknown field types, use a string value
            if !fieldname.is_empty() {
                Some(Value::String(format!("_Test {}", label)))
            } else {
                None
            }
        }
    }
}

fn generate_test_python_file(
    config: &Config,
    doctype: &str,
    snake_name: &str,
    dependencies: &[String],
) -> String {
    let class_name = to_pascalc(doctype);
    let current_year = Utc::now().format("%Y");

    let dependencies_str = if dependencies.is_empty() {
        "[]".to_string()
    } else {
        let deps: Vec<String> = dependencies.iter().map(|d| format!("\"{}\"", d)).collect();
        format!("[{}]", deps.join(", "))
    };

    format!(
        r#"# Copyright (c) {}, {}
# For license information, please see license.txt

import unittest

import frappe
from frappe.test_runner import make_test_records
from frappe.tests.utils import FrappeTestCase
from frappe.utils import add_months, flt, today

from {}.{}.doctype.{}.{} import {}

# Test dependencies
test_dependencies = {}

# Load test records
test_records = frappe.get_test_records("{}")


class Test{}(FrappeTestCase):
    def setUp(self):
        """Set up test data"""
        # @TODO: add setup code here if needed
        pass

    def tearDown(self):
        """Clean up test data"""
        # @TODO: add teardown code here if needed
        pass

    def test_creation(self):
        """Test basic document creation"""
        # @TODO: add test for document creation
        pass

    def test_validation(self):
        """Test document validation"""
        # @TODO: add test for document validation
        pass
"#,
        current_year,
        config.app_name,
        to_snakec(&config.app_name),
        to_snakec(&config.app_name),
        snake_name,
        snake_name,
        class_name,
        dependencies_str,
        doctype,
        class_name
    )
}

