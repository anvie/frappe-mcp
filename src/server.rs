#![allow(dead_code)]
use std::sync::Arc;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::functools;
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

// -----------------------------
// Args / DTOs
// -----------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindSymbolsArgs {
    /// Symbol name to search for across the project/app source files
    pub name: String,

    /// Search in: `backend`, `frontend`, `all` (default: all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_in: Option<String>,

    /// Whether to search using fuzzy matching (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy: Option<bool>,

    /// Maximum number of matches to return (default 5)
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
pub struct GetDoctypeArgs {
    /// DocType name (e.g., "Sales Invoice")
    pub name: String,

    /// Show DocType metadata only (default: false)
    /// This will read only the JSON metadata file if available.
    pub metadata_only: Option<bool>,
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

    /// find_symbols: search for a symbol across the project source files.
    #[tool(description = "Search for symbols across the app source files")]
    fn find_symbols(
        &self,
        Parameters(args): Parameters<FindSymbolsArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::find_symbols(
            &self.config,
            &self.anal,
            &args.name,
            args.search_in,
            args.fuzzy,
            args.limit,
        )
    }

    /// get_function_signature: get function signature from project code files by name,
    /// optionally within a specific module or including built-in Frappe modules.
    #[tool(description = "Try to extract a function signature from app source files")]
    fn get_function_signature(
        &self,
        Parameters(args): Parameters<GetFunctionSignatureArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::get_function_signature(
            &self.config,
            &self.anal,
            &args.name,
            args.module,
            args.builtin,
        )
    }

    /// get_doctype: get DocType information by name, eg: "Sales Invoice"
    #[tool(description = "Search and get a DocType information (by name) in the app")]
    fn get_doctype(
        &self,
        Parameters(args): Parameters<GetDoctypeArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::get_doctype(
            &self.config,
            &self.anal,
            &args.name,
            args.metadata_only.unwrap_or(false),
        )
    }

    /// Simple echo (handy for debugging)
    #[tool(description = "Echo back provided JSON params")]
    fn echo(&self, Parameters(object): Parameters<JsonObject>) -> Result<CallToolResult, McpError> {
        mcp_return!(serde_json::Value::Object(object).to_string())
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
                "Frappe Based Project Explorer server. Tools: find_symbols, get_function_signature, get_doctype, echo. Prompt: example_prompt."
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
                let memo = "\
                    Explorer Notes\n\n\
                    Use tools:\n\
                    - find_symbols { name, search_in?, fuzzy?, limit? }\n\
                    - get_function_signature { name, module?, builtin? }\n\
                    - get_doctype { name, metadata_only? }
                ";
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
        assert!(r.has_route("find_symbols"));
        assert!(r.has_route("get_function_signature"));
        assert!(r.has_route("get_doctype"));
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
