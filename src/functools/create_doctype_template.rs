#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldDefinition {
    pub fieldname: String,
    pub fieldtype: String,
    pub label: String,
    pub reqd: Option<bool>,
    pub options: Option<String>,
}

pub fn create_doctype_template(
    config: &Config,
    _anal: &AnalyzedData,
    name: &str,
    module: &str,
    fields: Option<Vec<FieldDefinition>>,
) -> McpResult {
    let snake_name = to_snakec(name);
    let doctype_dir = format!(
        "{}/{}/doctype/{}",
        config.app_absolute_path,
        module.to_lowercase(),
        snake_name
    );

    // Check if DocType already exists
    if Path::new(&doctype_dir).exists() {
        mcp_return!(format!(
            "DocType '{}' already exists at: {}",
            name, doctype_dir
        ));
    }

    // Create directory structure
    if let Err(e) = fs::create_dir_all(&doctype_dir) {
        mcp_return!(format!("Failed to create directory {}: {}", doctype_dir, e));
    }

    let mut result = Vec::new();

    // 1. Create JSON metadata file
    let json_content = create_json_metadata(name, &fields.unwrap_or_default());
    let json_path = format!("{}/{}.json", doctype_dir, snake_name);
    if let Err(e) = fs::write(&json_path, json_content) {
        mcp_return!(format!("Failed to write JSON file: {}", e));
    }
    result.push(format!("✓ Created metadata: {}", json_path));

    // 2. Create Python controller file
    let py_content = create_python_controller(name, &snake_name);
    let py_path = format!("{}/{}.py", doctype_dir, snake_name);
    if let Err(e) = fs::write(&py_path, py_content) {
        mcp_return!(format!("Failed to write Python file: {}", e));
    }
    result.push(format!("✓ Created controller: {}", py_path));

    // 3. Create JavaScript form file
    let js_content = create_javascript_form(name, &snake_name);
    let js_path = format!("{}/{}.js", doctype_dir, snake_name);
    if let Err(e) = fs::write(&js_path, js_content) {
        mcp_return!(format!("Failed to write JavaScript file: {}", e));
    }
    result.push(format!("✓ Created form script: {}", js_path));

    // 4. Create __init__.py file
    let init_path = format!("{}/__init__.py", doctype_dir);
    if let Err(e) = fs::write(&init_path, "") {
        mcp_return!(format!("Failed to write __init__.py: {}", e));
    }
    result.push(format!("✓ Created __init__.py: {}", init_path));

    let summary = format!(
        "DocType '{}' template created successfully in module '{}':\n\n{}\n\nNext steps:\n- Run 'bench migrate' to install the DocType\n- Customize fields in the JSON metadata\n- Add business logic in the Python controller",
        name,
        module,
        result.join("\n")
    );

    mcp_return!(summary)
}

fn create_json_metadata(name: &str, fields: &[FieldDefinition]) -> String {
    let mut default_fields = vec![FieldDefinition {
        fieldname: "naming_series".to_string(),
        fieldtype: "Select".to_string(),
        label: "Series".to_string(),
        reqd: Some(true),
        options: Some(format!(
            "{}-YYYY-MM-DD-####",
            name.chars().take(3).collect::<String>().to_uppercase()
        )),
    }];

    // Add custom fields if provided
    default_fields.extend_from_slice(fields);

    let json = serde_json::json!({
        "actions": [],
        "allow_copy": false,
        "allow_events_in_timeline": false,
        "allow_guest_to_view": false,
        "allow_import": true,
        "allow_rename": true,
        "autoname": "naming_series:",
        "beta": false,
        "creation": "2024-01-01 00:00:00.000000",
        "default_view": "List",
        "doctype": "DocType",
        "editable_grid": true,
        "engine": "InnoDB",
        "field_order": default_fields.iter().map(|f| &f.fieldname).collect::<Vec<_>>(),
        "fields": default_fields,
        "icon": "fa fa-file-text",
        "idx": 0,
        "in_create": false,
        "is_submittable": false,
        "is_tree": false,
        "issingle": false,
        "istable": false,
        "max_attachments": 0,
        "modified": "2024-01-01 00:00:00.000000",
        "modified_by": "Administrator",
        "module": name,
        "name": name,
        "naming_rule": "By \"Naming Series\" field",
        "owner": "Administrator",
        "permissions": [
            {
                "create": true,
                "delete": true,
                "email": true,
                "export": true,
                "print": true,
                "read": true,
                "report": true,
                "role": "System Manager",
                "share": true,
                "write": true
            }
        ],
        "quick_entry": false,
        "read_only": false,
        "read_only_onload": false,
        "show_name_in_global_search": false,
        "sort_field": "modified",
        "sort_order": "DESC",
        "states": [],
        "track_changes": true,
        "track_seen": false,
        "track_views": false
    });

    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

fn create_python_controller(name: &str, snake_name: &str) -> String {
    format!(
        r#"# Copyright (c) 2024, Your Company and contributors
# For license information, please see license.txt

import frappe
from frappe.model.document import Document


class {}(Document):
    """
    {} DocType Controller
    
    This class contains the business logic for the {} DocType.
    Add your custom methods and validations here.
    """
    
    def before_insert(self):
        """Called before inserting the document into the database."""
        pass
    
    def validate(self):
        """Called during document validation."""
        pass
    
    def before_save(self):
        """Called before saving the document."""
        pass
    
    def after_insert(self):
        """Called after inserting the document into the database."""
        pass
    
    def on_update(self):
        """Called after updating the document."""
        pass
    
    def on_trash(self):
        """Called when the document is being deleted."""
        pass
"#,
        snake_name
            .split('_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(""),
        name,
        name
    )
}

fn create_javascript_form(name: &str, _snake_name: &str) -> String {
    format!(
        r#"// Copyright (c) 2024, Your Company and contributors
// For license information, please see license.txt

frappe.ui.form.on('{}', {{
    refresh: function(frm) {{
        // Called when the form is loaded or refreshed
        
        // Example: Add custom button
        // frm.add_custom_button(__('Custom Action'), function() {{
        //     frappe.msgprint('Custom button clicked!');
        // }});
    }},
    
    validate: function(frm) {{
        // Called during form validation
        // Return false to prevent saving
    }},
    
    before_save: function(frm) {{
        // Called before saving the document
    }},
    
    after_save: function(frm) {{
        // Called after saving the document
    }}
}});
"#,
        name
    )
}

