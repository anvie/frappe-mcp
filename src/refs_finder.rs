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
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Occurrence {
    pub file: String,
    pub line: usize,
    pub var: String,
    pub kind: String, // "attr" | "subscript" | "get" | "set" | "append" | "get_value" | "inline"
}

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub struct DoctypeUsage {
    // field -> occurrences
    pub fields: BTreeMap<String, Vec<Occurrence>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Output {
    pub doctypes: BTreeMap<String, DoctypeUsage>,
    pub unknown: BTreeMap<String, BTreeMap<String, Vec<Occurrence>>>, // file -> field -> occurrences (doctype tak diketahui)
    pub stats: Stats,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Stats {
    pub files_scanned: usize,
    pub py_files: usize,
    pub doctypes_detected: usize,
    pub total_field_hits: usize,
}

pub fn analyze_frappe_field_usage(root: &str) -> Result<Output> {
    let root = Path::new(root);
    if !root.exists() {
        bail!("Root path does not exist: {}", root.display());
    }

    // Sebelumnya: let rx_bind_str = Regex::new(r#" ... "#).unwrap();
    let rx_bind_str = Regex::new(
        r#"(?x)
    (?P<var>[A-Za-z_]\w*)            # nama variabel di kiri
    \s*=\s*
    frappe\.(?P<fn1>get_doc|new_doc|get_cached_doc)
    \s*\(
        \s*
        (?:
            ["'](?P<dt1>[^"']+)["']                 # argumen doctype sebagai string
          |
            \{\s*["']doctype["']\s*:\s*["'](?P<dt2>[^"']+)["']  # atau dict dengan key doctype
        )
        [^)]*                                       # argumen tambahan apa pun
    \)                                              # TUTUP PAREN LITERAL (harus di-escape)
    "#,
    )
    .expect("rx_bind_str bad");

    let rx_inline_call = Regex::new(
        r#"(?x)
    frappe\.(?P<fn>get_doc|new_doc|get_cached_doc)\s*\(
        \s*(?:
            ["'](?P<dt_inline>[^"']+)["']
          |
            \{\s*["']doctype["']\s*:\s*["'](?P<dt_inline2>[^"']+)["']
        )
        [^)]*
    \)                                              # TUTUP PAREN LITERAL
    \.(?P<method>append|get|set|get_value)
    \s*\(\s*["'](?P<field>[^"']+)["']
    "#,
    )
    .expect("rx_inline_call bad");

    // Field access (we’ll run per known var name)
    // attr:   var.customer
    // sub:    var["customer"]
    // get:    var.get("customer")
    // set:    var.set("customer", ...)
    // append: var.append("items", {...})
    // get_value: var.get_value("field")
    let rx_attr_tpl = r#"(?x)\b{var}\.(?P<field>[A-Za-z_]\w*)\b"#;
    let rx_sub_tpl = r#"(?x)\b{var}\s*\[\s*["'](?P<field>[^"']+)["']\s*\]"#;
    let rx_get_tpl = r#"(?x)\b{var}\.get\s*\(\s*["'](?P<field>[^"']+)["']"#;
    let rx_set_tpl = r#"(?x)\b{var}\.set\s*\(\s*["'](?P<field>[^"']+)["']"#;
    let rx_app_tpl = r#"(?x)\b{var}\.append\s*\(\s*["'](?P<field>[^"']+)["']"#;
    let rx_gv_tpl = r#"(?x)\b{var}\.get_value\s*\(\s*["'](?P<field>[^"']+)["']"#;

    let mut out = Output {
        doctypes: BTreeMap::new(),
        unknown: BTreeMap::new(),
        stats: Stats::default(),
    };

    let mut files_scanned = 0usize;
    let mut py_files = 0usize;
    let mut doctypes_detected: BTreeSet<String> = BTreeSet::new();
    let mut total_hits = 0usize;

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        files_scanned += 1;

        if path.extension().and_then(|e| e.to_str()) != Some("py") {
            continue;
        }
        py_files += 1;

        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let lines: Vec<&str> = content.lines().collect();

        let pstr = normalize_sep(path);
        let primary_dt = infer_primary_doctype_from_path(path); // Some(dt) jika di dalam doctype/<dt>/<dt>.py

        if primary_dt.is_some() {
            // get DocType name from it json file
            let json_file = path.with_extension("json");
            // loking for pattern like: `"name": "Sales Invoice",`
            if json_file.exists() && json_file.is_file() {
                if let Ok(json_content) = fs::read_to_string(&json_file) {
                    let rx_dt_name =
                        Regex::new(r#""name"\s*:\s*"([^"]+)""#).expect("rx_dt_name bad");
                    if let Some(caps) = rx_dt_name.captures(&json_content) {
                        if let Some(m) = caps.get(1) {
                            let dt_name = m.as_str();
                            scan_type_hints_in_doctype_py(
                                &pstr,
                                dt_name,
                                &content,
                                &mut out,
                                &mut total_hits,
                            );
                        }
                    }
                }
            }
        }

        // 1) Temukan binding var -> doctype (satu file)
        let mut var_to_dt: BTreeMap<String, String> = BTreeMap::new();
        for (_i, line) in lines.iter().enumerate() {
            for cap in rx_bind_str.captures_iter(line) {
                let var = cap.name("var").unwrap().as_str().to_string();
                let dt = cap
                    .name("dt1")
                    .map(|m| m.as_str())
                    .or_else(|| cap.name("dt2").map(|m| m.as_str()))
                    .unwrap_or("")
                    .to_string();
                if !dt.is_empty() {
                    var_to_dt.insert(var, dt.clone());
                    doctypes_detected.insert(dt);
                }
            }
        }

        // 2) Kumpulkan field usage dari var yang diketahui tipenya
        for (var, dt) in var_to_dt.clone() {
            let rx_attr = Regex::new(&rx_attr_tpl.replace("{var}", &regex::escape(&var))).unwrap();
            let rx_sub = Regex::new(&rx_sub_tpl.replace("{var}", &regex::escape(&var))).unwrap();
            let rx_get = Regex::new(&rx_get_tpl.replace("{var}", &regex::escape(&var))).unwrap();
            let rx_set = Regex::new(&rx_set_tpl.replace("{var}", &regex::escape(&var))).unwrap();
            let rx_app = Regex::new(&rx_app_tpl.replace("{var}", &regex::escape(&var))).unwrap();
            let rx_gv = Regex::new(&rx_gv_tpl.replace("{var}", &regex::escape(&var))).unwrap();

            for (ln, line) in lines.iter().enumerate() {
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "attr",
                    &rx_attr,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "subscript",
                    &rx_sub,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "get",
                    &rx_get,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "set",
                    &rx_set,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "append",
                    &rx_app,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    &var,
                    "get_value",
                    &rx_gv,
                    line,
                    &mut total_hits,
                );
            }
        }

        // 3) Inline call: frappe.get_doc("X", ...).append("items", ...)
        for (ln, line) in lines.iter().enumerate() {
            for cap in rx_inline_call.captures_iter(line) {
                let dt = cap
                    .name("dt_inline")
                    .map(|m| m.as_str())
                    .or_else(|| cap.name("dt_inline2").map(|m| m.as_str()))
                    .unwrap_or("")
                    .to_string();
                let field = cap.name("field").unwrap().as_str().to_string();
                if !dt.is_empty() && !field.is_empty() {
                    let entry = out.doctypes.entry(dt.clone()).or_default();
                    entry.fields.entry(field).or_default().push(Occurrence {
                        file: pstr.clone(),
                        line: ln + 1,
                        var: "<inline>".into(),
                        kind: "inline".into(),
                    });
                    doctypes_detected.insert(dt);
                    total_hits += 1;
                }
            }
        }

        // 4) Heuristik untuk `doc` tanpa tipe:
        //    Jika file ini di doctype/<dt>/<dt>.py, maka asumsikan var 'doc' bertipe dt.
        //    Scan akses field dari 'doc'.
        if let Some(dt) = primary_dt {
            let var = "doc";
            let rx_attr = Regex::new(&rx_attr_tpl.replace("{var}", var)).unwrap();
            let rx_sub = Regex::new(&rx_sub_tpl.replace("{var}", var)).unwrap();
            let rx_get = Regex::new(&rx_get_tpl.replace("{var}", var)).unwrap();
            let rx_set = Regex::new(&rx_set_tpl.replace("{var}", var)).unwrap();
            let rx_app = Regex::new(&rx_app_tpl.replace("{var}", var)).unwrap();
            let rx_gv = Regex::new(&rx_gv_tpl.replace("{var}", var)).unwrap();

            for (ln, line) in lines.iter().enumerate() {
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "attr",
                    &rx_attr,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "subscript",
                    &rx_sub,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "get",
                    &rx_get,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "set",
                    &rx_set,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "append",
                    &rx_app,
                    line,
                    &mut total_hits,
                );
                collect_hits(
                    &mut out.doctypes,
                    &dt,
                    &pstr,
                    ln + 1,
                    var,
                    "get_value",
                    &rx_gv,
                    line,
                    &mut total_hits,
                );
            }
        } else {
            // Skip for now
            continue;
            // Kalau tidak bisa infer doctype, simpan sebagai unknown (per file) untuk 'doc'
            #[allow(unreachable_code)]
            let var = "doc";
            let rx_attr = Regex::new(&rx_attr_tpl.replace("{var}", var)).unwrap();
            let rx_sub = Regex::new(&rx_sub_tpl.replace("{var}", var)).unwrap();
            let rx_get = Regex::new(&rx_get_tpl.replace("{var}", var)).unwrap();
            let rx_set = Regex::new(&rx_set_tpl.replace("{var}", var)).unwrap();
            let rx_app = Regex::new(&rx_app_tpl.replace("{var}", var)).unwrap();
            let rx_gv = Regex::new(&rx_gv_tpl.replace("{var}", var)).unwrap();

            for (ln, line) in lines.iter().enumerate() {
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "attr",
                    &rx_attr,
                    line,
                    &mut total_hits,
                );
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "subscript",
                    &rx_sub,
                    line,
                    &mut total_hits,
                );
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "get",
                    &rx_get,
                    line,
                    &mut total_hits,
                );
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "set",
                    &rx_set,
                    line,
                    &mut total_hits,
                );
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "append",
                    &rx_app,
                    line,
                    &mut total_hits,
                );
                collect_unknown(
                    &mut out.unknown,
                    &pstr,
                    ln + 1,
                    var,
                    "get_value",
                    &rx_gv,
                    line,
                    &mut total_hits,
                );
            }
        }
    }

    out.stats.files_scanned = files_scanned;
    out.stats.py_files = py_files;
    out.stats.doctypes_detected = doctypes_detected.len();
    out.stats.total_field_hits = total_hits;

    // let json = serde_json::to_string_pretty(&out)?;
    // Ok(json)
    Ok(out)
}

fn collect_hits(
    doctypes: &mut BTreeMap<String, DoctypeUsage>,
    dt: &str,
    file: &str,
    line: usize,
    var: &str,
    kind: &str,
    rx: &Regex,
    text: &str,
    total_hits: &mut usize,
) {
    for cap in rx.captures_iter(text) {
        if let Some(fm) = cap.name("field") {
            let field = fm.as_str().to_string();
            let usage = doctypes.entry(dt.to_string()).or_default();
            usage.fields.entry(field).or_default().push(Occurrence {
                file: file.to_string(),
                line,
                var: var.to_string(),
                kind: kind.to_string(),
            });
            *total_hits += 1;
        }
    }
}

fn collect_unknown(
    unknown: &mut BTreeMap<String, BTreeMap<String, Vec<Occurrence>>>,
    file: &str,
    line: usize,
    var: &str,
    kind: &str,
    rx: &Regex,
    text: &str,
    total_hits: &mut usize,
) {
    for cap in rx.captures_iter(text) {
        if let Some(fm) = cap.name("field") {
            let field = fm.as_str().to_string();
            unknown
                .entry(file.to_string())
                .or_default()
                .entry(field)
                .or_default()
                .push(Occurrence {
                    file: file.to_string(),
                    line,
                    var: var.to_string(),
                    kind: kind.to_string(),
                });
            *total_hits += 1;
        }
    }
}

fn normalize_sep(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s
    }
}

/// Infer primary doctype from a path matching: .../doctype/<dt>/<dt>.py
/// - Works with absolute or relative paths
/// - Doesn't assume a fixed index for "doctype"
/// - Picks the *nearest* (rightmost) doctype/<dt>/<dt>.py segment if multiple exist
pub fn infer_primary_doctype_from_path(path: &Path) -> Option<String> {
    // Collect components as OsStr slices (skip things like RootDir/Prefix semantics)
    let parts: Vec<_> = path.iter().collect();

    // Walk from right to left so we prefer the deepest/nearest match
    // Need at least 3 components: doctype / <dt> / <dt>.py
    for i in (0..parts.len()).rev() {
        if i + 2 >= parts.len() {
            continue;
        }
        // Require literal "doctype"
        if parts[i] == "doctype" {
            let dt_dir = parts[i + 1];
            let file = parts[i + 2];

            let file_path = Path::new(file);
            // Only consider Python files (*.py); relax this if you want other extensions
            if file_path.extension().and_then(|e| e.to_str()) != Some("py") {
                continue;
            }
            if let Some(stem) = file_path.file_stem() {
                if stem == dt_dir {
                    return Some(dt_dir.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

fn scan_type_hints_in_doctype_py(
    pstr: &str,
    dt_name: &str,
    content_raw: &str,
    out: &mut Output,
    total_hits: &mut usize,
) {
    // // Only run for app/**/doctype/<dt>/<dt>.py to know which DocType we’re populating.
    // let Some(dt) = infer_primary_doctype_from_path(path) else {
    //     return;
    // };

    // Normalize newlines
    let content = content_raw.replace("\r\n", "\n");
    let lines: Vec<&str> = content.lines().collect();

    // Helpers
    let leading_ws = |s: &str| s.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    // let is_blank_or_comment = |s: &str| {
    //     let t = s.trim_start();
    //     t.is_empty() || t.starts_with('#')
    // };
    let contains_document_base = |bases: &str| -> bool {
        // Accept any qualified "Document" in the bases list: Document, frappe.model...Document, etc.
        bases.contains("Document")
    };

    // Find all class blocks using a simple state machine (no regex).
    // A "class" header looks like: [indent] class Name(bases...):
    #[allow(dead_code)]
    #[derive(Clone)]
    struct ClassBlock {
        name: String,
        header_line: usize, // 1-based
        indent: usize,
        body_start_line: usize, // first line after header
        body_end_line: usize,   // exclusive
    }
    let mut classes: Vec<ClassBlock> = Vec::new();

    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        // println!("Line {}: {}", i + 1, line);
        let trimmed = line.trim_start();
        if trimmed.starts_with("class ") {
            // Parse header
            let indent = leading_ws(line);
            let header_line = i + 1;

            // Extract class name and base list (loose but safe)
            // e.g. class Allowance(frappe.model.document.Document):
            // or    class Invoice(Document, AccountsController):
            let mut name = String::new();
            let mut bases = String::new();

            if let Some(after_class) = trimmed.strip_prefix("class ") {
                if let Some(colon_idx) = after_class.find(':') {
                    let header_inside = &after_class[..colon_idx]; // Name(...bases...)
                    if let Some(paren_idx) = header_inside.find('(') {
                        name = header_inside[..paren_idx].trim().to_string();
                        if let Some(rparen_idx) = header_inside.rfind(')') {
                            bases = header_inside[paren_idx + 1..rparen_idx].trim().to_string();
                        }
                    } else {
                        // Rare: "class Name:" without bases
                        name = header_inside.trim().to_string();
                        bases.clear();
                    }
                }
            }

            // Only consider classes that (likely) represent doctypes (include Document in bases if present)
            if bases.is_empty() || contains_document_base(&bases) {
                // Determine end of class body by indentation
                let body_start = i + 1;
                let mut j = body_start;
                while j < lines.len() {
                    let l = lines[j];
                    if l.trim().is_empty() {
                        j += 1;
                        continue;
                    }
                    let ind = leading_ws(l);
                    if ind <= indent {
                        break; // dedent => class body ends
                    }
                    j += 1;
                }
                classes.push(ClassBlock {
                    name,
                    header_line,
                    indent,
                    body_start_line: body_start + 1, // 1-based line index of first body line
                    body_end_line: j + 1,            // exclusive, 1-based
                });
                i = j;
                continue;
            }
        }
        i += 1;
    }

    // Pattern to match a type-hinted field line.
    // Examples it accepts:
    //   saldo: DF.Currency
    //   bank_account: DF.Link | None
    //   status: DF.Literal["Active", "Suspended"]
    let field_from_typehint = |s: &str| -> Option<(String, String)> {
        // Remove trailing inline comment safely
        let mut core = s;
        if let Some(hash) = s.find('#') {
            core = &s[..hash];
        }
        let core = core.trim();
        // Must contain colon
        let colon = core.find(':')?;
        let (lhs, rhs) = core.split_at(colon);
        let lhs = lhs.trim();
        let rhs = rhs[1..].trim(); // skip ':'

        // LHS must be a valid identifier
        if !lhs
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        {
            return None;
        }
        if !lhs.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return None;
        }

        // RHS must start with DF.
        let rhs = rhs.trim_start();
        let rhs = if let Some(after) = rhs.strip_prefix("DF.") {
            after
        } else {
            return None;
        };

        // Take the primary annotation token (Currency, Link, Literal[...], etc.)
        // Stop at whitespace or union bar.
        let mut ann = String::new();
        for ch in rhs.chars() {
            if ch.is_whitespace() || ch == '|' {
                break;
            }
            ann.push(ch);
        }
        if ann.is_empty() {
            return None;
        }
        Some((lhs.to_string(), ann))
    };

    // Scan inside each class body for either:
    //   (A) begin/end markers, or
    //   (B) if TYPE_CHECKING / if typing.TYPE_CHECKING block(s)
    for class in classes {
        // Slice the class body (1-based to 0-based)
        let start_idx = class.body_start_line.saturating_sub(1);
        let end_idx = class.body_end_line.saturating_sub(1).min(lines.len());
        if start_idx >= end_idx {
            continue;
        }

        // First try (A) comment markers
        let mut begin_idx: Option<usize> = None;
        let mut end_idx_marker: Option<usize> = None;
        for k in start_idx..end_idx {
            let t = lines[k].trim_start();
            if t.starts_with("# begin: auto-generated types") {
                begin_idx = Some(k + 1);
            } else if t.starts_with("# end: auto-generated types") {
                end_idx_marker = Some(k);
                break;
            }
        }

        let mut consumed_any = false;

        if let (Some(bi), Some(ei)) = (begin_idx, end_idx_marker) {
            for ln in bi..ei {
                if let Some((field, ann)) = field_from_typehint(lines[ln]) {
                    let usage = out.doctypes.entry(dt_name.to_string()).or_default();
                    usage
                        .fields
                        .entry(field.clone())
                        .or_default()
                        .push(Occurrence {
                            file: pstr.to_string(),
                            line: ln + 1,
                            var: class.name.clone(),
                            kind: format!("typehint:DF.{ann}"),
                        });
                    *total_hits += 1;
                    consumed_any = true;
                }
            }
        }

        // // Then (B) TYPE_CHECKING blocks (there can be multiple)
        // // We only parse them if we didn’t find the comment block, or to collect extra hints.
        // let mut k = start_idx;
        // while k < end_idx {
        //     let line = lines[k];
        //     let trimmed = line.trim_start();
        //     if trimmed.starts_with("if TYPE_CHECKING:")
        //         || trimmed.starts_with("if typing.TYPE_CHECKING:")
        //     {
        //         let block_indent = leading_ws(line);
        //         // The following lines with strictly greater indentation belong to the block
        //         let mut m = k + 1;
        //         // Find the first non-empty line to set base indent inside block
        //         let mut inner_base: Option<usize> = None;
        //         let mut tmp = m;
        //         while tmp < end_idx {
        //             let l = lines[tmp];
        //             if l.trim().is_empty() {
        //                 tmp += 1;
        //                 continue;
        //             }
        //             inner_base = Some(leading_ws(l));
        //             break;
        //         }
        //         let inner_base = inner_base.unwrap_or(block_indent + 4);
        //
        //         while m < end_idx {
        //             let l = lines[m];
        //             if !l.trim().is_empty() && leading_ws(l) < inner_base {
        //                 break; // dedented → end of TYPE_CHECKING block
        //             }
        //             if let Some((field, ann)) = field_from_typehint(l) {
        //                 let usage = out.doctypes.entry(dt_name.to_string()).or_default();
        //                 usage
        //                     .fields
        //                     .entry(field.clone())
        //                     .or_default()
        //                     .push(Occurrence {
        //                         file: pstr.to_string(),
        //                         line: m + 1,
        //                         var: class.name.clone(),
        //                         kind: format!("typehint:DF.{ann}"),
        //                     });
        //                 *total_hits += 1;
        //                 consumed_any = true;
        //             }
        //             m += 1;
        //         }
        //         k = m;
        //         continue;
        //     }
        //     k += 1;
        // }
        //
        // If still nothing consumed and you want to be even more forgiving,
        // you can scan the entire class body for `field: DF.*` lines (commented out by default).
        if !consumed_any {
            // Uncomment if you want this fallback:
            // for ln in start_idx..end_idx {
            //     if let Some((field, ann)) = field_from_typehint(lines[ln]) {
            //         let usage = out.doctypes.entry(dt.clone()).or_default();
            //         usage.fields.entry(field.clone()).or_default().push(Occurrence {
            //             file: pstr.to_string(),
            //             line: ln + 1,
            //             var: class.name.clone(),
            //             kind: format!("typehint:DF.{ann}"),
            //         });
            //         *total_hits += 1;
            //     }
            // }
        }
    }
}
