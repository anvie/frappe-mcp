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
use serde::Deserialize;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::serdeutil::deserialize_bool_from_int_or_bool;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

#[derive(Deserialize)]
struct DocField {
    pub fieldname: String,
    pub fieldtype: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub options: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub reqd: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub unique: Option<bool>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub read_only: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub hidden: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub in_list_view: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub in_standard_filter: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub in_global_search: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub search_index: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_from_int_or_bool")]
    pub bold: Option<bool>,
    #[serde(default)]
    pub precision: Option<u8>,
    #[serde(default)]
    pub depends_on: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
struct DocTypeStruct {
    pub default_view: String,

    #[serde(
        rename = "istable",
        default,
        deserialize_with = "deserialize_bool_from_int_or_bool"
    )]
    pub is_child: Option<bool>,

    #[serde(
        default,
        rename = "issingle",
        deserialize_with = "deserialize_bool_from_int_or_bool"
    )]
    pub is_single: Option<bool>,

    pub fields: Vec<DocField>,
}

pub fn get_doctype(config: &Config, anal: &AnalyzedData, name: &str, json_only: bool) -> McpResult {
    let target = name;
    let mut result: Vec<String> = Vec::new();

    let candidate = anal
        .doctypes
        .iter()
        .find(|a| a.name.to_lowercase() == target.to_lowercase());

    if candidate.is_none() {
        // try snake_case variant
        let target_snake = to_snakec(target);
        let candidate_snake = anal
            .doctypes
            .iter()
            .find(|a| a.name.to_lowercase() == target_snake.to_lowercase());
        if candidate_snake.is_some() {
            result.push(format!(
                "Note: DocType '{}' not found, but '{}' (snake_case) found",
                target, target_snake
            ));
        } else {
            result.push(format!("DocType '{}' not found", target));
        }
        mcp_return!(result.join("\n"));
    }
    let doc = candidate.unwrap();

    if json_only {
        if doc.meta_file.is_none() {
            mcp_return!(format!(
                "DocType '{}' found, but has no metadata file",
                doc.name
            ));
        } else {
            // read whole metadata file
            let meta_path = format!(
                "{}/{}",
                config.app_absolute_path,
                doc.meta_file.as_ref().unwrap()
            );
            if !Path::new(&meta_path).exists() {
                mcp_return!(format!(
                    "DocType '{}' metadata file '{}' not found",
                    target, meta_path
                ));
            }
            let content = std::fs::read_to_string(meta_path).unwrap_or_else(|_| "".to_string());
            mcp_return!(format!("{}", content));
        }
    }

    result.push(format!("DocType '{}' found:\n", doc.name));
    result.push(format!("- Module: {}", doc.module));
    result.push(format!("- Backend: {}", doc.backend_file));
    if let Some(front) = &doc.frontend_file {
        result.push(format!("- Frontend: {}", front));
    }
    if let Some(meta_file) = &doc.meta_file {
        result.push(format!("- Metadata: {}", meta_file));
    }

    let root = &config.app_absolute_path;

    if doc.meta_file.is_some() {
        let json_file = format!("{}/{}", root, doc.meta_file.as_ref().unwrap());
        // deserialize json file to get more info
        tracing::debug!("Parsing DocType metadata from {}", json_file);
        if !Path::new(&json_file).exists() {
            result.push(format!("  Note: Metadata file '{}' not found", json_file));
            mcp_return!(result.join("\n"));
        }
        if let Ok(doc_struct) = parse_doctype_metadata(&json_file) {
            result.push("\n## Basic Structure".to_string());
            result.push(format!("- Default View: {}", doc_struct.default_view));
            if let Some(is_single) = doc_struct.is_single {
                result.push(format!("- Is Single: {}", is_single));
            }
            if let Some(is_child) = doc_struct.is_child {
                result.push(format!("- Is Child Table: {}", is_child));
            }
            result.push("- Fields:".to_string());
            for field in doc_struct.fields {
                result.push(format!(
                    "  - {} - \"{}\" ({}){}",
                    &field.fieldname,
                    field.label.as_ref().unwrap_or(&field.fieldname),
                    field.fieldtype,
                    if field.reqd.unwrap_or(false) {
                        " [Required]"
                    } else {
                        ""
                    }
                ));
            }
        }
    }
    let out = if result.is_empty() {
        format!("DocType '{}' not found under '{}'", target, root)
    } else {
        result.join("\n")
    };

    mcp_return!(out)

    // let target_pyname = to_snakec(&target);
    // let candidate = format!("{}/{}", root, target_pyname);
    //
    // // direct relative candidate
    // if !Path::new(&candidate).exists() {
    //     mcp_return!(format!("DocType '{}' not found under '{}'", target, root));
    // }
    // let mut json_file = String::new();
    //
    // // full-tree search for both file name and in-file markers
    // for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
    //     if !entry.file_type().is_file() {
    //         continue;
    //     }
    //     let p = entry.path();
    //     let path_str = p.display().to_string();
    //
    //     if path_str.ends_with(&format!("{}/{}.json", target_pyname, target_pyname))
    //         && path_str.contains("/doctype/")
    //     {
    //         json_file = path_str.clone();
    //         result.push(format!("- Metadata: {}", path_str));
    //         continue;
    //     }
    //
    //     if path_str.ends_with(&format!("{}/{}.js", target_pyname, target_pyname))
    //         && path_str.contains("/doctype/")
    //     {
    //         result.push(format!("- Frontend: {}", path_str));
    //         continue;
    //     }
    //
    //     if path_str.ends_with(&format!("{}/{}.py", target_pyname, target_pyname))
    //         && path_str.contains("/doctype/")
    //     {
    //         result.push(format!("- Backend: {}", path_str));
    //         continue;
    //     }
    // }
    // tracing::debug!(
    //     "get_doctype: candidate '{}', json_file '{}'",
    //     candidate,
    //     json_file
    // );
    // println!(
    //     "println |> get_doctype: candidate '{}', json_file '{}'",
    //     candidate, json_file
    // );
}

fn parse_doctype_metadata(json_file: &str) -> Result<DocTypeStruct, McpError> {
    if !Path::new(json_file).exists() {
        return Err(McpError::new(
            ErrorCode::INVALID_REQUEST,
            "Metadata file not found",
            Some(serde_json::json!({ "file": json_file })),
        ));
    }
    parse_doctype_metadata_string(&std::fs::read_to_string(json_file).map_err(|e| {
        McpError::new(
            ErrorCode::INVALID_REQUEST,
            "Failed to read metadata file",
            Some(serde_json::json!({ "file": json_file, "error": e.to_string() })),
        )
    })?)
}

fn parse_doctype_metadata_string(json_content: &str) -> Result<DocTypeStruct, McpError> {
    let doc_struct: DocTypeStruct = serde_json::from_str(json_content).map_err(|e| {
        McpError::new(
            ErrorCode::INVALID_REQUEST,
            "Failed to parse metadata JSON",
            Some(serde_json::json!({ "error": e.to_string() })),
        )
    })?;
    Ok(doc_struct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doctype_metadata() {
        let test_json = r#"
        {
            "default_view": "List",
            "fields": [
                {
                    "fieldname": "name",
                    "fieldtype": "Data",
                    "label": "Name",
                    "reqd": true
                },
                {
                    "fieldname": "description",
                    "fieldtype": "Text",
                    "label": "Description"
                }
            ]
        }
        "#;
        let temp_file = "/tmp/test_doctype.json";
        std::fs::write(temp_file, test_json).unwrap();
        let result = parse_doctype_metadata(temp_file);
        assert!(result.is_ok());
        let doc_struct = result.unwrap();
        assert_eq!(doc_struct.default_view, "List");
        assert_eq!(doc_struct.fields.len(), 2);
        assert_eq!(doc_struct.fields[0].fieldname, "name");
        assert_eq!(doc_struct.fields[0].reqd.unwrap(), true);
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_parse_doctype_metadata_string_invalid() {
        let invalid_json = r#"{ "default_view": "List", "fields": [ { "fieldname": "name" } ] "#; // Missing closing braces
        let result = parse_doctype_metadata_string(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_doctype_metadata_string_with_test_data() {
        let test_content = include_str!("../../test_data/branch.json");

        let result = parse_doctype_metadata_string(&test_content);
        assert!(
            result.is_ok(),
            "Failed to parse doctype metadata: {:?}",
            result.err()
        );

        let doc_struct = result.unwrap();
        assert_eq!(doc_struct.default_view, "List");
        assert_eq!(doc_struct.fields.len(), 12);

        // Test specific fields from branch.json
        let branch_code_field = &doc_struct.fields[0];
        assert_eq!(branch_code_field.fieldname, "branch_code");
        assert_eq!(branch_code_field.fieldtype, "Data");
        assert_eq!(branch_code_field.reqd, Some(true));
        assert_eq!(branch_code_field.unique, Some(true));
        assert_eq!(branch_code_field.in_list_view, Some(true));

        let branch_name_field = &doc_struct.fields[1];
        assert_eq!(branch_name_field.fieldname, "branch_name");
        assert_eq!(branch_name_field.fieldtype, "Data");
        assert_eq!(branch_name_field.reqd, Some(true));
        assert_eq!(branch_name_field.in_list_view, Some(true));

        // Test Link field with options
        let country_field = &doc_struct
            .fields
            .iter()
            .find(|f| f.fieldname == "country")
            .unwrap();
        assert_eq!(country_field.fieldtype, "Link");
        assert_eq!(country_field.options, Some("Country".to_string()));
        assert_eq!(country_field.reqd, Some(true));

        // Test Check field with default value
        let is_active_field = &doc_struct
            .fields
            .iter()
            .find(|f| f.fieldname == "is_active")
            .unwrap();
        assert_eq!(is_active_field.fieldtype, "Check");
        assert_eq!(is_active_field.default, Some("0".to_string()));

        // Test field with specific options (phone/email)
        let phone_field = &doc_struct
            .fields
            .iter()
            .find(|f| f.fieldname == "phone")
            .unwrap();
        assert_eq!(phone_field.fieldtype, "Data");
        assert_eq!(phone_field.options, Some("Phone".to_string()));

        let email_field = &doc_struct
            .fields
            .iter()
            .find(|f| f.fieldname == "email")
            .unwrap();
        assert_eq!(email_field.fieldtype, "Data");
        assert_eq!(email_field.options, Some("Email".to_string()));

        // Verify field ordering matches expected order from field_order
        let expected_order = vec![
            "branch_code",
            "branch_name",
            "address",
            "city",
            "country",
            "postal_code",
            "phone",
            "email",
            "branch_manager",
            "is_active",
            "opening_date",
            "province",
        ];

        for (i, expected_fieldname) in expected_order.iter().enumerate() {
            assert_eq!(doc_struct.fields[i].fieldname, *expected_fieldname);
        }

        // Test that integer boolean values are properly converted to bool
        assert_eq!(branch_code_field.reqd, Some(true)); // was 1 in JSON
        assert_eq!(branch_code_field.unique, Some(true)); // was 1 in JSON
        assert_eq!(branch_code_field.in_list_view, Some(true)); // was 1 in JSON

        // Test that missing boolean fields default to None
        assert_eq!(branch_code_field.hidden, None);
        assert_eq!(branch_code_field.read_only, None);
    }
}
