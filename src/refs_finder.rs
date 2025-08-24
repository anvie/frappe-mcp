use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
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

        // 1) Temukan binding var -> doctype (satu file)
        let mut var_to_dt: BTreeMap<String, String> = BTreeMap::new();
        for (i, line) in lines.iter().enumerate() {
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

// Infer primary doctype dari pola path:  .../doctype/<dt>/<dt>.py
fn infer_primary_doctype_from_path(path: &Path) -> Option<String> {
    // contoh: app/accounts/doctype/sales_invoice/sales_invoice.py
    let parts: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    for i in 0..parts.len().saturating_sub(3) {
        if parts[i] == "doctype" {
            let dt_dir = &parts[i + 1];
            let file = &parts[i + 2];
            // file = "<dt>.py" ?
            if let Some(stem) = Path::new(file).file_stem().and_then(|s| s.to_str()) {
                if stem == dt_dir {
                    return Some(dt_dir.to_string());
                }
            }
        }
    }
    None
}
