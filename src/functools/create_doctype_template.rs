#![allow(dead_code)]
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::{to_camelc, to_snakec};
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
    let camel_name = to_camelc(name);
    let doctype_dir = format!(
        "{}/{}/{}/doctype/{}",
        config.app_absolute_path,
        snake_name,
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

    let fields = fields.unwrap_or_default();

    // 1. Create JSON metadata file
    let json_content = create_json_metadata(name, &fields);
    let json_path = format!("{}/{}.json", doctype_dir, snake_name);
    if let Err(e) = fs::write(&json_path, json_content) {
        mcp_return!(format!("Failed to write JSON file: {}", e));
    }
    result.push(format!("✓ Created metadata: {}", json_path));

    // 2. Create Python controller file
    let py_content = create_python_controller(config, name, &camel_name, &fields);
    let py_path = format!("{}/{}.py", doctype_dir, snake_name);
    if let Err(e) = fs::write(&py_path, py_content) {
        mcp_return!(format!("Failed to write Python file: {}", e));
    }
    result.push(format!("✓ Created controller: {}", py_path));

    // 3. Create JavaScript form file
    let js_content = create_javascript_form(config, name, &snake_name);
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

fn get_current_year() -> i32 {
    Utc::now().year()
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
        "creation": format!("{}-01-01 00:00:00.000000", get_current_year()),
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
        "modified": format!("{}-01-01 00:00:00.000000", get_current_year()),
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

/// Generate Python type hints for fields, example output:
///    # begin: auto-generated types
///    # This code is auto-generated. Do not modify anything in this block.
///
///    from typing import TYPE_CHECKING
///
///    if TYPE_CHECKING:
///        from frappe.types import DF
///
///        bank_account: DF.Link | None
///        card_number: DF.Data | None
///        status: DF.Literal["Active", "Suspended", "Disabled"]
///        unique_code: DF.Data | None
///    # end: auto-generated types
///
fn generate_field_types(fields: &[FieldDefinition]) -> String {
    let mut types = Vec::new();
    for field in fields {
        let py_type = match field.fieldtype.as_str() {
            "Data" | "Small Text" | "Text" | "Text Editor" | "Code" | "Password" | "Attach"
            | "Attach Image" | "Dynamic Link" => "DF.Data",
            "Link" => "DF.Link",
            "Select" => {
                if let Some(options) = &field.options {
                    if options.contains('\n') {
                        // Multi-line options, probably not a DocType reference
                        "DF.Data"
                    } else {
                        // Single line, could be a DocType reference
                        "DF.Literal[...]"
                    }
                } else {
                    "DF.Data"
                }
            }
            "Int" => "DF.Int",
            "Float" => "DF.Float",
            "Currency" => "DF.Currency",
            "Percent" => "DF.Percent",
            "Check" => "DF.Check",
            "Date" => "DF.Date",
            "Datetime" => "DF.Datetime",
            "Time" => "DF.Time",
            _ => "DF.Data", // Default to Data for unknown types
        };
        let optional = if field.reqd.unwrap_or(false) {
            ""
        } else {
            " | None"
        };
        types.push(format!("{}: {}{}", field.fieldname, py_type, optional));
    }
    types.join("\n        ")
}

fn create_python_controller(
    config: &Config,
    name: &str,
    camel_name: &str,
    fields: &[FieldDefinition],
) -> String {
    let df_types = generate_field_types(fields);

    format!(
        r#"# Copyright (c) {}, {}
# For license information, please see license.txt

import frappe
from frappe.model.document import Document


class {}(Document):
    """
    {} DocType Controller
    
    This class contains the business logic for the {} DocType.
    Add your custom methods and validations here.
    """
    # begin: auto-generated types
    # This code is auto-generated. Do not modify anything in this block.

    from typing import TYPE_CHECKING

    if TYPE_CHECKING:
        from frappe.types import DF

        {}
    # end: auto-generated types
    
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
        get_current_year(),
        config.app_name,
        camel_name,
        name,
        name,
        df_types
    )
}

fn create_javascript_form(config: &Config, name: &str, _snake_name: &str) -> String {
    format!(
        r#"// Copyright (c) {}, {}
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
        get_current_year(),
        config.app_name,
        name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_field_types_basic_types() {
        let fields = vec![
            FieldDefinition {
                fieldname: "title".to_string(),
                fieldtype: "Data".to_string(),
                label: "Title".to_string(),
                reqd: Some(true),
                options: None,
            },
            FieldDefinition {
                fieldname: "description".to_string(),
                fieldtype: "Text".to_string(),
                label: "Description".to_string(),
                reqd: Some(false),
                options: None,
            },
            FieldDefinition {
                fieldname: "amount".to_string(),
                fieldtype: "Currency".to_string(),
                label: "Amount".to_string(),
                reqd: Some(true),
                options: None,
            },
        ];

        let result = generate_field_types(&fields);
        let expected =
            "title: DF.Data\n        description: DF.Data | None\n        amount: DF.Currency";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_field_types_numeric_and_date() {
        let fields = vec![
            FieldDefinition {
                fieldname: "count".to_string(),
                fieldtype: "Int".to_string(),
                label: "Count".to_string(),
                reqd: Some(true),
                options: None,
            },
            FieldDefinition {
                fieldname: "rate".to_string(),
                fieldtype: "Float".to_string(),
                label: "Rate".to_string(),
                reqd: Some(false),
                options: None,
            },
            FieldDefinition {
                fieldname: "created_date".to_string(),
                fieldtype: "Date".to_string(),
                label: "Created Date".to_string(),
                reqd: Some(true),
                options: None,
            },
            FieldDefinition {
                fieldname: "start_time".to_string(),
                fieldtype: "Time".to_string(),
                label: "Start Time".to_string(),
                reqd: Some(false),
                options: None,
            },
        ];

        let result = generate_field_types(&fields);
        let expected = "count: DF.Int\n        rate: DF.Float | None\n        created_date: DF.Date\n        start_time: DF.Time | None";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_field_types_boolean_and_percent() {
        let fields = vec![
            FieldDefinition {
                fieldname: "is_active".to_string(),
                fieldtype: "Check".to_string(),
                label: "Is Active".to_string(),
                reqd: Some(true),
                options: None,
            },
            FieldDefinition {
                fieldname: "discount".to_string(),
                fieldtype: "Percent".to_string(),
                label: "Discount".to_string(),
                reqd: Some(false),
                options: None,
            },
        ];

        let result = generate_field_types(&fields);
        let expected = "is_active: DF.Check\n        discount: DF.Percent | None";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_field_types_link_and_select() {
        let fields = vec![
            FieldDefinition {
                fieldname: "customer".to_string(),
                fieldtype: "Link".to_string(),
                label: "Customer".to_string(),
                reqd: Some(true),
                options: Some("Customer".to_string()),
            },
            FieldDefinition {
                fieldname: "status".to_string(),
                fieldtype: "Select".to_string(),
                label: "Status".to_string(),
                reqd: Some(true),
                options: Some("Draft\n        Submitted\n        Cancelled".to_string()),
            },
            FieldDefinition {
                fieldname: "priority".to_string(),
                fieldtype: "Select".to_string(),
                label: "Priority".to_string(),
                reqd: Some(false),
                options: Some("High".to_string()),
            },
        ];

        let result = generate_field_types(&fields);
        let expected =
            "customer: DF.Link\n        status: DF.Data\n        priority: DF.Literal[...] | None";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_field_types_unknown_type() {
        let fields = vec![FieldDefinition {
            fieldname: "unknown_field".to_string(),
            fieldtype: "CustomType".to_string(),
            label: "Unknown Field".to_string(),
            reqd: Some(true),
            options: None,
        }];

        let result = generate_field_types(&fields);
        let expected = "unknown_field: DF.Data";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_field_types_empty() {
        let fields = vec![];
        let result = generate_field_types(&fields);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_field_types_all_field_types() {
        let fields = vec![
            FieldDefinition {
                fieldname: "data_field".to_string(),
                fieldtype: "Data".to_string(),
                label: "Data Field".to_string(),
                reqd: Some(true),
                options: None,
            },
            FieldDefinition {
                fieldname: "small_text_field".to_string(),
                fieldtype: "Small Text".to_string(),
                label: "Small Text Field".to_string(),
                reqd: Some(false),
                options: None,
            },
            FieldDefinition {
                fieldname: "text_editor_field".to_string(),
                fieldtype: "Text Editor".to_string(),
                label: "Text Editor Field".to_string(),
                reqd: Some(false),
                options: None,
            },
            FieldDefinition {
                fieldname: "datetime_field".to_string(),
                fieldtype: "Datetime".to_string(),
                label: "Datetime Field".to_string(),
                reqd: Some(true),
                options: None,
            },
        ];

        let result = generate_field_types(&fields);
        let expected = "data_field: DF.Data\n        small_text_field: DF.Data | None\n        text_editor_field: DF.Data | None\n        datetime_field: DF.Datetime";
        assert_eq!(result, expected);
    }
}
