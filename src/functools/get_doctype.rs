#![allow(dead_code)]
use serde::Deserialize;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};
use walkdir::WalkDir;

type McpResult = Result<CallToolResult, McpError>;

#[derive(Deserialize)]
struct DocField {
    pub fieldname: String,
    pub fieldtype: String,
    pub label: Option<String>,
    pub options: Option<String>,
    pub reqd: Option<bool>,
    pub unique: Option<bool>,
    pub default: Option<String>,
    pub read_only: Option<bool>,
    pub hidden: Option<bool>,
    pub in_list_view: Option<bool>,
    pub in_standard_filter: Option<bool>,
    pub in_global_search: Option<bool>,
    pub search_index: Option<bool>,
    pub bold: Option<bool>,
    pub precision: Option<u8>,
    pub depends_on: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
struct DocTypeStruct {
    pub default_view: String,
    pub fields: Vec<DocField>,
}

pub fn get_doctype(config: &Config, anal: &AnalyzedData, name: &str, json_only: bool) -> McpResult {
    let target = name;
    let mut result: Vec<String> = Vec::new();

    let candidate = anal
        .doctypes
        .iter()
        .find(|a| a.name.to_lowercase() == target.to_lowercase());

    if let Some(doc) = candidate {
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

        let mut msg = format!("DocType '{}' found:\n\n", doc.name);
        msg.push_str(&format!("- Module: {}\n", doc.module));
        msg.push_str(&format!("- Backend: {}\n", doc.backend_file));
        if let Some(front) = &doc.frontend_file {
            msg.push_str(&format!("- Frontend: {}\n", front));
        }
        if let Some(meta_file) = &doc.meta_file {
            msg.push_str(&format!("- Metadata: {}\n", meta_file));
        }
        mcp_return!(msg);
    }

    let target_pyname = to_snakec(&target);
    let root = &config.app_absolute_path;
    let candidate = format!("{}/{}", root, target_pyname);

    // direct relative candidate
    if !Path::new(&candidate).exists() {
        mcp_return!(format!("DocType '{}' not found under '{}'", target, root));
    }
    let mut json_file = String::new();

    // full-tree search for both file name and in-file markers
    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let path_str = p.display().to_string();

        if path_str.ends_with(&format!("{}/{}.json", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            json_file = path_str.clone();
            result.push(format!("- Metadata: {}", path_str));
            continue;
        }

        if path_str.ends_with(&format!("{}/{}.js", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            result.push(format!("- Frontend: {}", path_str));
            continue;
        }

        if path_str.ends_with(&format!("{}/{}.py", target_pyname, target_pyname))
            && path_str.contains("/doctype/")
        {
            result.push(format!("- Backend: {}", path_str));
            continue;
        }
    }

    if !json_file.is_empty() {
        // deserialize json file to get more info
        if let Ok(doc_struct) = parse_doctype_metadata(&json_file) {
            result.push("\n## Basic Structure".to_string());
            result.push(format!("- Default View: {}", doc_struct.default_view));
            result.push(format!("- Fields:"));
            for field in doc_struct.fields {
                result.push(format!(
                    "  - {} ({}){}",
                    field.label.unwrap_or(field.fieldname),
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
        format!("DocType '{}' found:\n{}", target, result.join("\n"))
    };

    mcp_return!(out)
}

pub fn parse_doctype_metadata(json_file: &str) -> Result<DocTypeStruct, McpError> {
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

pub fn parse_doctype_metadata_string(json_content: &str) -> Result<DocTypeStruct, McpError> {
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
    use crate::analyze::AnalyzedData;
    use crate::config::Config;

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
}
