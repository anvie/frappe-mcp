#![allow(dead_code)]
use std::{fs, io::BufRead, path::Path, sync::Arc};

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::fileutil::{match_func_signature_in_file, to_pyname};
use regex::Regex;
use rmcp::{
    handler::server::{router::prompt::PromptRouter, tool::ToolRouter, wrapper::Parameters},
    model::*,
    prompt, prompt_handler, prompt_router, schemars,
    service::RequestContext,
    tool,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use rmcp::{tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use walkdir::{DirEntry, WalkDir};

// -----------------------------
// Args / DTOs
// -----------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindReferencesArgs {
    /// Symbol to search for across the project
    pub symbol: String,
    /// Project root directory (defaults to current working dir)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// File extensions to scan, e.g., ["rs","ts","js","py"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exts: Option<Vec<String>>,
    /// Maximum number of matches to return (default 200)
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFunctionSignatureArgs {
    /// Function name to find
    pub name: String,

    /// Module name to search in (optional)
    /// if not set will search in all available modules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// Search in Frappe's built-in modules (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindDoctypeArgs {
    /// DocType name (e.g., "Sales Invoice")
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExamplePromptArgs {
    /// A message to put in the prompt
    pub message: String,
}

// -----------------------------
// Server impl
// -----------------------------

#[derive(Clone)]
pub struct ProjectExplorer {
    _state_counter: Arc<Mutex<i32>>, // example state (unused but shows pattern)
    tool_router: ToolRouter<ProjectExplorer>,
    prompt_router: PromptRouter<ProjectExplorer>,
    config: Config,
    anal: AnalyzedData,
}

#[tool_router]
#[prompt_router]
impl ProjectExplorer {
    pub fn new(config: Config, anal: AnalyzedData) -> Self {
        Self {
            _state_counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            config,
            anal,
        }
    }

    fn create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    // -------------------------
    // Tools
    // -------------------------

    /// find_references: naive, fast grep over text files by extension
    #[tool(description = "Search for a symbol across the project tree")]
    fn find_references(
        &self,
        Parameters(args): Parameters<FindReferencesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = args.root.unwrap_or_else(|| ".".to_string());
        let limit = args.limit.unwrap_or(200);
        let exts = args.exts.unwrap_or_else(|| {
            vec![
                "rs", "ts", "tsx", "js", "jsx", "py", "toml", "json", "yaml", "yml",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect()
        });

        let mut results = Vec::new();
        let symbol = args.symbol;

        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if !exts.iter().any(|x| x == ext) {
                    continue;
                }
            } else {
                continue;
            }

            // Read line by line and match substring
            if let Ok(file) = fs::File::open(entry.path()) {
                let reader = std::io::BufReader::new(file);
                for (idx, line) in reader.lines().enumerate() {
                    if results.len() >= limit {
                        break;
                    }
                    if let Ok(text) = line {
                        if text.contains(&symbol) {
                            let p = entry.path().display().to_string();
                            results.push(format!("{p}:{}: {}", idx + 1, text.trim()));
                        }
                    }
                }
            }
            if results.len() >= limit {
                break;
            }
        }

        let out = if results.is_empty() {
            format!("No references for '{}' under '{}'", symbol, root)
        } else {
            format!(
                "References for '{}' ({} results):\n{}",
                symbol,
                results.len(),
                results.join("\n")
            )
        };

        Ok(CallToolResult::success(vec![Content::text(out)]))
    }

    /// get_function_signature: naive regex for Rust/TS/JS/Python
    #[tool(description = "Try to extract a function signature from source files")]
    fn get_function_signature(
        &self,
        Parameters(args): Parameters<GetFunctionSignatureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let name = args.name;
        let module = args.module.unwrap_or_default();
        let builtin = args.builtin.unwrap_or(false);

        let exts = vec!["py", "js"];

        let mut matches = Vec::new();

        if module != "" {
            let f_mod = self
                .anal
                .modules
                .iter()
                .find(|m| m.name == module)
                .ok_or_else(|| {
                    McpError::invalid_request("module_not_found", Some(json!({ "module": module })))
                })?;
            let candidate = format!("{}/{}", self.config.app_absolute_path, f_mod.location);
            tracing::info!("Searching in module path: {}", candidate);

            if Path::new(&candidate).exists() && Path::new(&candidate).is_dir() {
                for entry in WalkDir::new(&candidate).into_iter().filter_map(|e| e.ok()) {
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                        if !exts.iter().any(|x| x == &ext) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                        continue;
                    }
                    if matches.len() > 2 {
                        break;
                    }
                }
            } else {
                let out = format!(
                    "Module path '{}' does not exist or is not a directory",
                    candidate
                );
                return Ok(CallToolResult::success(vec![Content::text(out)]));
            }
        }

        if builtin {
            for entry in WalkDir::new(&format!("{}/apps/frappe", self.config.frappe_bench_dir))
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if !exts.iter().any(|x| x == &ext) {
                        continue;
                    }
                } else {
                    continue;
                }
                if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                    continue;
                }
                if matches.len() > 2 {
                    break;
                }
            }
        }

        if matches.is_empty() {
            for entry in WalkDir::new(&self.config.app_absolute_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if !exts.iter().any(|x| x == &ext) {
                        continue;
                    }
                } else {
                    continue;
                }

                if !match_func_signature_in_file(&name, &entry, &mut matches)? {
                    continue;
                }

                if matches.len() > 2 {
                    break;
                }
            }
        }

        let out = if matches.is_empty() {
            format!(
                "No signature for '{}' found under '{}' (exts: {:?})",
                name, "??", exts
            )
        } else {
            format!(
                "Found signature(s) for '{}' in {} location(s):\n{}",
                name,
                matches.len(),
                matches
                    .iter()
                    .map(|a| format!("- {}", a))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        };

        Ok(CallToolResult::success(vec![Content::text(out)]))
    }

    /// find_doctype: look for Frappe DocType information
    #[tool(description = "Search for a DocType (by name) in the project")]
    fn find_doctype(
        &self,
        Parameters(args): Parameters<FindDoctypeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let target = args.name;

        let mut hits: Vec<String> = Vec::new();

        let candidate = self
            .anal
            .doctypes
            .iter()
            .find(|a| a.name.to_lowercase() == target.to_lowercase());

        if let Some(doc) = candidate {
            let mut msg = format!("DocType '{}' found:\n\n", target);
            msg.push_str(&format!("- Module: {}\n", doc.module));
            msg.push_str(&format!("- Backend: {}\n", doc.backend_file));
            if let Some(front) = &doc.frontend_file {
                msg.push_str(&format!("- Frontend: {}\n", front));
            }
            if let Some(meta_file) = &doc.meta_file {
                msg.push_str(&format!("- Metadata: {}\n", meta_file));
            }
            return Ok(CallToolResult::success(vec![Content::text(msg)]));
        }

        let target_pyname = to_pyname(&target);
        let root = &self.config.app_absolute_path;
        let candidate = format!("{}/{}", root, target_pyname);

        // direct relative candidate
        if !Path::new(&candidate).exists() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "DocType '{}' not found under '{}'",
                target, root
            ))]));
        }

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
                hits.push(format!("- Metadata: {}", path_str));
                continue;
            }

            if path_str.ends_with(&format!("{}/{}.js", target_pyname, target_pyname))
                && path_str.contains("/doctype/")
            {
                hits.push(format!("- Frontend: {}", path_str));
                continue;
            }

            if path_str.ends_with(&format!("{}/{}.py", target_pyname, target_pyname))
                && path_str.contains("/doctype/")
            {
                hits.push(format!("- Backend: {}", path_str));
                continue;
            }
        }

        let out = if hits.is_empty() {
            format!("DocType '{}' not found under '{}'", target, root)
        } else {
            format!("DocType '{}' found:\n{}", target, hits.join("\n"))
        };

        Ok(CallToolResult::success(vec![Content::text(out)]))
    }

    /// Simple echo (handy for debugging)
    #[tool(description = "Echo back provided JSON params")]
    fn echo(&self, Parameters(object): Parameters<JsonObject>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::Value::Object(object).to_string(),
        )]))
    }

    // -------------------------
    // Example prompts (optional)
    // -------------------------

    /// example_prompt: tiny demo prompt to show prompt routing works
    #[prompt(name = "example_prompt")]
    async fn example_prompt(
        &self,
        Parameters(args): Parameters<ExamplePromptArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let prompt = format!("Example prompt message: '{}'", args.message);
        Ok(vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::text(prompt),
        }])
    }
}

// -----------------------------
// ServerHandler impl
// -----------------------------

#[tool_handler]
#[prompt_handler]
impl ServerHandler for ProjectExplorer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Project Explorer server. Tools: find_references, get_function_signature, find_doctype, echo. Prompt: example_prompt."
                    .to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        // Advertise CWD and a sample memo
        Ok(ListResourcesResult {
            resources: vec![
                self.create_resource_text("cwd:///", "Current Working Directory"),
                self.create_resource_text("memo://explorer-notes", "Explorer Notes"),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "cwd:///" => {
                let cwd = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string());
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(&cwd, uri)],
                })
            }
            "memo://explorer-notes" => {
                let memo = "Explorer Notes\n\nUse tools:\n- find_references { symbol, root?, exts?, limit? }\n- get_function_signature { name, language?, root?, exts? }\n- find_doctype { name, root? }";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({ "uri": uri })),
            )),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize via HTTP transport");
        }
        Ok(self.get_info())
    }
}

// -----------------------------
// Main: run over stdio
// -----------------------------

// #[tokio::main]
pub async fn run(config: Config) -> anyhow::Result<()> {
    // Pretty logs help when debugging with a local MCP client
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::debug!("Load analyzed data: anazyled_output.toml");
    let anal = AnalyzedData::from_file("analyzed_output.toml")
        .map(|data| {
            tracing::debug!("Analyzed Data: {:#?}", data);
            data
        })
        .unwrap_or_else(|e| {
            tracing::warn!(
            "Failed to load analyzed data from 'analyzed_output.toml': {}. Using empty analysis.",
            e
            );
            AnalyzedData {
                doctypes: Vec::new(),
                modules: Vec::new(),
            }
        });
    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = ProjectExplorer::new(config, anal)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;

    Ok(())
}

// -----------------------------
// Tests (quick sanity)
// -----------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routers_have_tools() {
        let r = ProjectExplorer::tool_router();
        assert!(r.has_route("find_references"));
        assert!(r.has_route("get_function_signature"));
        assert!(r.has_route("find_doctype"));
        assert!(r.has_route("echo"));
    }

    #[tokio::test]
    async fn prompt_has_route() {
        let r = ProjectExplorer::prompt_router();
        assert!(r.has_route("example_prompt"));
        let attr = ProjectExplorer::example_prompt_prompt_attr();
        assert_eq!(attr.name, "example_prompt");
    }
}
