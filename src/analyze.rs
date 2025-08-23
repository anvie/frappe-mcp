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

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocType {
    pub name: String,
    pub backend_file: String,
    pub frontend_file: Option<String>,
    pub meta_file: Option<String>,
    pub module: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub name: String,
    pub location: String,
}

#[derive(Serialize, Deserialize)]
struct Analysis {
    doctypes: Vec<DocType>,
    modules: Vec<Module>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AnalyzedData {
    pub doctypes: Vec<DocType>,
    pub modules: Vec<Module>,
}

impl AnalyzedData {
    pub fn from_toml_str(toml_str: &str) -> Result<AnalyzedData, toml::de::Error> {
        toml::from_str(toml_str)
    }

    pub fn from_file(file_path: &str) -> Result<AnalyzedData, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        let data = Self::from_toml_str(&content)?;
        Ok(data)
    }
}

pub fn analyze_frappe_app(
    root: &str,
    relative_path: &str,
    output_file: &str,
) -> anyhow::Result<()> {
    let root_path = Path::new(root);
    let leaf = root_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let root_sub_path = root_path.join(leaf);
    let modules_txt = root_sub_path.join("modules.txt");

    println!("Modules file: {:?}", modules_txt);

    // Read modules.txt
    let file = fs::File::open(&modules_txt)?;
    let reader = BufReader::new(file);
    let mut modules = Vec::new();
    let mut doctypes = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let module_title = line.trim();
        if module_title.is_empty() {
            continue;
        }

        let module_dir = module_title.to_lowercase();
        let module_path = root_sub_path.join(&module_dir);

        if module_path.exists() && module_path.is_dir() {
            modules.push(Module {
                name: module_title.to_string(),
                location: to_relative_path(
                    &module_path.to_string_lossy().to_string(),
                    &root_sub_path.to_string_lossy().to_string(),
                    relative_path,
                ),
            });

            // scan doctypes
            let doctype_path = module_path.join("doctype");
            if doctype_path.exists() && doctype_path.is_dir() {
                for entry in fs::read_dir(&doctype_path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let doctype_name = entry.file_name().to_string_lossy().to_string();
                        if doctype_name.is_empty() {
                            continue;
                        }
                        if ["__pycache__", ".git"].contains(&doctype_name.as_str()) {
                            continue;
                        }
                        let doctype_dir = entry.path();

                        let backend_file = doctype_dir.join(format!("{}.py", &doctype_name));
                        let frontend_file = doctype_dir.join(format!("{}.js", &doctype_name));
                        let meta_file = doctype_dir.join(format!("{}.json", &doctype_name));

                        doctypes.push(DocType {
                            name: capitalize_words(&doctype_name),
                            backend_file: to_relative_path(
                                &backend_file.to_string_lossy().to_string(),
                                &root_sub_path.to_string_lossy().to_string(),
                                relative_path,
                            ),
                            frontend_file: if frontend_file.exists() {
                                Some(to_relative_path(
                                    &frontend_file.to_string_lossy().to_string(),
                                    &root_sub_path.to_string_lossy().to_string(),
                                    relative_path,
                                ))
                            } else {
                                None
                            },
                            meta_file: if meta_file.exists() {
                                Some(to_relative_path(
                                    &meta_file.to_string_lossy().to_string(),
                                    &root_sub_path.to_string_lossy().to_string(),
                                    relative_path,
                                ))
                            } else {
                                None
                            },
                            module: module_title.to_string(),
                        });
                    }
                }
            }
        }
    }

    let analysis = Analysis { doctypes, modules };

    // // Serialize to TOML (exact structure: top-level doctypes/modules)
    // let mut toml_str = String::new();
    // toml_str.push_str("doctypes = [\n");
    // for d in &analysis.doctypes {
    //     toml_str.push_str(&format!(
    //         "  {{ name = \"{}\", backend_file = \"{}\", frontend_file = {}, module = \"{}\" }},\n",
    //         d.name,
    //         d.backend_file,
    //         d.frontend_file
    //             .as_ref()
    //             .map(|s| format!("\"{}\"", s))
    //             .unwrap_or("\"\"".to_string()),
    //         d.module
    //     ));
    // }
    // toml_str.push_str("]\n\n");
    //
    // toml_str.push_str("modules = [\n");
    // for m in &analysis.modules {
    //     toml_str.push_str(&format!(
    //         "  {{ name = \"{}\", location = \"{}\" }},\n",
    //         m.name, m.location
    //     ));
    // }
    // toml_str.push_str("]\n");
    //

    let toml_str = toml::to_string(&analysis)?;
    fs::write(output_file, toml_str)?;

    Ok(())
}

fn to_relative_path(full_path: &str, base_path: &str, relative_path: &str) -> String {
    if let Some(pos) = full_path.find(base_path) {
        let rel_path = &full_path[pos + base_path.len()..];
        format!("{}{}", relative_path, rel_path)
    } else {
        full_path.to_string()
    }
}

fn capitalize_words(s: &str) -> String {
    s.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_words() {
        assert_eq!(capitalize_words("school"), "School");
        assert_eq!(capitalize_words("school_management"), "School Management");
        assert_eq!(
            capitalize_words("school management system"),
            "School Management System"
        );
        assert_eq!(capitalize_words(""), "");
    }
}
