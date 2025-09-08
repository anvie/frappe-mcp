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
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::functools;
use crate::{analyze::AnalyzedData, stringutil::to_snakec};
use rmcp::{
    handler::server::{router::prompt::PromptRouter, tool::ToolRouter, wrapper::Parameters},
    model::*,
    prompt_handler, prompt_router, schemars,
    service::RequestContext,
    tool,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use rmcp::{tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

    /// Maximum number of matches to return (default 50)
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

    /// When true, return only the JSON content of the DocType
    pub json_only: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateDoctypeTemplateArgs {
    /// DocType name (e.g., "Task")
    pub name: String,

    /// Target module name (e.g., "Projects")
    pub module: String,

    /// Optional field definitions for the DocType
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldDefinition>>,

    /// Whether the DocType is a single instance (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_single: Option<bool>,

    /// Whether the DocType is a tree structure (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tree: Option<bool>,

    /// Whether the DocType is submittable (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_submittable: Option<bool>,

    /// Whether the DocType is a child table (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_child_table: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema, Clone)]
pub struct FieldDefinition {
    /// Field name (snake_case)
    pub fieldname: String,

    /// Field type (e.g., "Data", "Text", "Link", "Select")
    pub fieldtype: String,

    /// Field label for display
    pub label: String,

    /// Whether field is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reqd: Option<bool>,

    /// Options for Select/Link fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<String>,

    /// Whether to include field in list view
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_list_view: Option<bool>,

    /// Whether to include field in filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_standard_filter: Option<bool>,

    /// Whether field is read-only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Length for Data fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunTestsArgs {
    /// Specific module to test (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// Specific DocType to test (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctype: Option<String>,

    /// Specific test to run, e.g., "test_method_name" (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeLinksArgs {
    /// DocType name to analyze relationships for
    pub doctype: String,

    /// Maximum depth for relationship traversal (default: 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWebPageArgs {
    /// File path where the web page should be created, don't include "www/" prefix, eg: "about.html" or "info/contact.html"
    pub path: String,

    /// Page title (optional, defaults to filename)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Whether to include a basic CSS file (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_css: Option<bool>,

    /// Whether to include a basic JavaScript file (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_js: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFieldUsageArgs {
    /// DocType name to search field usage in
    pub doctype: String,

    /// Field name to search for usage
    pub field_name: String,

    /// Maximum number of occurrences to return (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExamplePromptArgs {
    /// A message to put in the prompt
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RunBenchCommandArgs {
    /// Arguments to pass to the `bench` command, eg: `migrate`, `mariadb -e "SELECT 1"`, etc.
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RunMariadbCommandArgs {
    /// SQL query to execute via bench mariadb command
    pub sql: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetDoctypeDbSchemaArgs {
    /// DocType name (e.g., "Sales Invoice")
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RunBenchExecuteArgs {
    /// Frappe function to execute, e.g., "frappe.db.get_list"
    pub frappe_function: String,

    /// Arguments to pass to the function (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,

    /// Keyword arguments to pass to the function (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kwargs: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTestTemplateArgs {
    /// DocType name (e.g., "Sales Invoice")
    pub doctype: String,

    /// List of dependency DocTypes for testing (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctype_dependencies: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDoctypesArgs {
    /// Optional module filter to list DocTypes only from a specific module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateReportTemplateArgs {
    /// Report name (e.g., "Sales Analysis")
    pub report_name: String,

    /// Target module name (e.g., "Sales")
    pub module: String,

    /// Report type: Script Report, Query Report, Report Builder (default: Script Report)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_type: Option<String>,

    /// Reference DocType for the report (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_doctype: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchFrappeDocsArgs {
    /// Search query string
    pub query: String,

    /// Filter by category (e.g., "doctypes", "api", "tutorial")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Use fuzzy search (default: true)
    #[serde(default = "default_true")]
    pub fuzzy: bool,

    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_true() -> bool {
    true
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadFrappeDocArgs {
    /// Document ID (e.g., "a7b9c3", "d8f2e1")
    pub id: String,
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
    anal: Arc<Mutex<AnalyzedData>>,
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
            anal: Arc::new(Mutex::new(anal)),
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
        let anal = self.anal.lock().unwrap();
        functools::find_symbols(
            &self.config,
            &anal,
            &args.name,
            args.search_in,
            args.fuzzy,
            args.limit,
        )
    }

    ///// get_function_signature: get function signature from project code files by name,
    ///// optionally within a specific module or including built-in Frappe modules.
    //#[tool(description = "Try to extract a function signature from app source files")]
    //fn get_function_signature(
    //    &self,
    //    Parameters(args): Parameters<GetFunctionSignatureArgs>,
    //) -> Result<CallToolResult, McpError> {
    //    let anal = self.anal.lock().unwrap();
    //    functools::get_function_signature(
    //        &self.config,
    //        &anal,
    //        &args.name,
    //        args.module,
    //        args.builtin,
    //    )
    //}

    /// get_doctype: get DocType information by name, eg: "Sales Invoice"
    #[tool(description = "Search and get a DocType information (by name) in the app")]
    fn get_doctype(
        &self,
        Parameters(args): Parameters<GetDoctypeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::get_doctype(
            &self.config,
            &anal,
            &args.name,
            args.json_only.unwrap_or(false),
        )
    }

    /// create_doctype_template: Generate boilerplate DocType structure
    #[tool(
        description = "Generate boilerplate DocType structure with JSON metadata, Python controller, and JS form files"
    )]
    fn create_doctype_template(
        &self,
        Parameters(args): Parameters<CreateDoctypeTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut anal = self.anal.lock().unwrap();
        functools::create_doctype_template(
            &self.config,
            &mut anal,
            &args.name,
            &args.module,
            args.fields.map(|fields| {
                fields
                    .into_iter()
                    .map(|f| functools::FieldDefinition {
                        fieldname: f.fieldname,
                        fieldtype: f.fieldtype,
                        label: f.label,
                        reqd: f.reqd.map(|a| if a { 1 } else { 0 }),
                        options: f.options,
                        length: f.length,
                        in_list_view: f.in_list_view.map(|a| if a { 1 } else { 0 }),
                        in_standard_filter: f.in_standard_filter.map(|a| if a { 1 } else { 0 }),
                        read_only: f.read_only.map(|a| if a { 1 } else { 0 }),
                    })
                    .collect()
            }),
            Some(functools::DoctypeSettings {
                is_single: args.is_single.unwrap_or(false),
                is_tree: args.is_tree.unwrap_or(false),
                is_submittable: args.is_submittable.unwrap_or(false),
                is_child_table: args.is_child_table.unwrap_or(false),
            }),
        )
    }

    /// run_tests: Execute unit tests for specific modules or doctypes
    #[tool(
        description = "Execute unit tests for specific modules, DocTypes, or entire app using bench run-tests"
    )]
    fn run_tests(
        &self,
        Parameters(args): Parameters<RunTestsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::run_tests(&self.config, &anal, args.module, args.doctype, args.test)
    }

    /// analyze_links: Map relationships between DocTypes
    #[tool(
        description = "Analyze and map relationships between DocTypes by examining Link, Table, and Select fields"
    )]
    fn analyze_links(
        &self,
        Parameters(args): Parameters<AnalyzeLinksArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::analyze_links(&self.config, &anal, &args.doctype, args.depth)
    }

    /// create_web_page: Generate boilerplate web page files with HTML, CSS, and JavaScript
    #[tool(
        description = "Generate boilerplate web page files with HTML, CSS, and JavaScript structure"
    )]
    fn create_web_page(
        &self,
        Parameters(args): Parameters<CreateWebPageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::create_web_page(
            &self.config,
            &anal,
            &args.path,
            args.title,
            args.include_css,
            args.include_js,
        )
    }

    /// find_field_usage: Search for references to a specific field within a DocType
    #[tool(
        description = "Search for references to a specific field of a DocType, this can help identify where a field is used in code"
    )]
    fn find_field_usage(
        &self,
        Parameters(args): Parameters<GetFieldUsageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::find_field_usage(
            &self.config,
            &anal,
            &args.doctype,
            &args.field_name,
            args.limit,
        )
    }

    /// run_bench_command: Run arbitrary `bench` command with arguments, e.g: `migrate`
    #[tool(
        description = "Run arbitrary bench command with args, e.g: `migrate`, the `--site` is auto-added, no need to include it."
    )]
    fn run_bench_command(
        &self,
        Parameters(args): Parameters<RunBenchCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::run_bench_command(
            &self.config,
            &self.anal.lock().unwrap(),
            &args.args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        )
    }

    /// get_doctype_db_schema: Get the database table schema for a specific DocType
    #[tool(
        description = "Get the database table schema for a specific DocType, this will execute SQL query into the database."
    )]
    fn get_doctype_db_schema(
        &self,
        Parameters(args): Parameters<GetDoctypeDbSchemaArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::get_doctype_db_schema(&self.config, &self.anal.lock().unwrap(), &args.name)
    }

    /// run_db_command: Execute SQL query via bench mariadb command
    #[tool(description = "Execute SQL query via bench mariadb command")]
    fn run_db_command(
        &self,
        Parameters(args): Parameters<RunMariadbCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::run_db_command(&self.config, &self.anal.lock().unwrap(), &args.sql)
    }

    /// bench_execute: Execute Frappe function via bench execute command
    #[tool(
        description = "Execute Frappe function via bench execute command with optional args and kwargs.\n\
        You don't need to escape quotes inside args.\n\
        Example: bench_execute(frappe.db.get_list, Invoice, {fields:[\"invoice_code\"]})"
    )]
    fn bench_execute(
        &self,
        Parameters(args): Parameters<RunBenchExecuteArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::bench_execute(
            &self.config,
            &self.anal.lock().unwrap(),
            &args.frappe_function,
            args.args.as_deref(),
            args.kwargs.as_deref(),
        )
    }

    /// search_frappe_docs: Search embedded Frappe documentation
    #[tool(
        description = "Search through Frappe framework documentation. Supports fuzzy and exact search, category filtering, and returns relevant snippets."
    )]
    fn search_frappe_docs(
        &self,
        Parameters(args): Parameters<SearchFrappeDocsArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::search_frappe_docs(&args.query, args.category, args.fuzzy, args.limit)
    }

    /// read_frappe_doc: Read a specific Frappe documentation file
    #[tool(
        description = "Read the full content of a specific Frappe documentation by its ID (e.g., 'a7b9c3', 'd8f2e1'). Use search_frappe_docs to find document IDs."
    )]
    fn read_frappe_doc(
        &self,
        Parameters(args): Parameters<ReadFrappeDocArgs>,
    ) -> Result<CallToolResult, McpError> {
        functools::get_frappe_doc(&args.id)
    }

    /// create_test_template: Create test template files for a Frappe DocType
    #[tool(description = "Create test template files for a Frappe DocType. \
            The function creates comprehensive test scaffolding, proper imports, FrappeTestCase inheritance, setUp/tearDown methods, and dependency declarations.")]
    fn create_test_template(
        &self,
        Parameters(args): Parameters<CreateTestTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut anal = self.anal.lock().unwrap();
        functools::create_test_template(
            &self.config,
            &mut anal,
            &args.doctype,
            args.doctype_dependencies,
        )
    }

    /// create_report_template: Create report template files for a Frappe Report
    #[tool(
        description = "Create report template files for starting with Frappe Report including Python logic file (.py), JavaScript filters (.js), JSON metadata (.json). \
            Creates a complete report structure with sample filters, columns, and data processing logic."
    )]
    fn create_report_template(
        &self,
        Parameters(args): Parameters<CreateReportTemplateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut anal = self.anal.lock().unwrap();
        functools::create_report_template(
            &self.config,
            &mut anal,
            &args.report_name,
            &args.module,
            args.report_type,
            args.ref_doctype,
        )
    }

    /// list_doctypes: List all available DocTypes in the current Frappe app
    #[tool(
        description = "List all available DocTypes in the current Frappe app, optionally filtered by module"
    )]
    fn list_doctypes(
        &self,
        Parameters(args): Parameters<ListDoctypesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let anal = self.anal.lock().unwrap();
        functools::list_doctypes(&self.config, &anal, args.module)
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
                "Frappe Based Project Explorer server. Tools: find_symbols, get_function_signature, get_doctype, list_doctypes, create_doctype_template, create_report_template, create_test_template, create_web_page, run_tests, analyze_links, find_field_usage, echo. Prompt: example_prompt."
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
                    - get_doctype { name, json_only? }\n\
                    - list_doctypes { module? }\n\
                    - create_doctype_template { name, module, fields? }\n\
                    - create_report_template { report_name, module, report_type?, ref_doctype? }\n\
                    - create_test_template { doctype, doctype_dependencies? }\n\
                    - create_web_page { path, title?, include_css?, include_js? }\n\
                    - run_tests { module?, doctype?, test_type? }\n\
                    - analyze_links { doctype, depth? }\n\
                    - find_field_usage { doctype, field_name, limit? }
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
// Helper functions
// -----------------------------

fn should_run_analysis(config: &Config, analysis_file: &str) -> bool {
    use std::path::Path;

    // Always analyze if file doesn't exist
    if !Path::new(analysis_file).exists() {
        tracing::info!(
            "Analysis file '{}' doesn't exist, will run analysis",
            analysis_file
        );
        return true;
    }

    // Check if any source files are newer than analysis file
    let app_path = Path::new(&config.app_absolute_path);
    if !app_path.exists() {
        tracing::warn!("App directory '{}' doesn't exist", config.app_absolute_path);
        return false;
    }

    // Get analysis file modification time
    let analysis_mtime = match std::fs::metadata(analysis_file).and_then(|m| m.modified()) {
        Ok(time) => time,
        Err(_) => {
            tracing::info!("Could not get analysis file modification time, will run analysis");
            return true;
        }
    };

    // Check if modules.txt is newer
    let modules_txt = app_path.join(&config.app_relative_path).join("modules.txt");
    if let Ok(metadata) = std::fs::metadata(&modules_txt) {
        if let Ok(mtime) = metadata.modified() {
            if mtime > analysis_mtime {
                tracing::info!("modules.txt is newer than analysis file, will run analysis");
                return true;
            }
        }
    }

    // Check if any doctype files are newer than analysis
    if check_doctype_files_newer(&config, analysis_mtime) {
        tracing::info!("Found doctype files newer than analysis file, will run analysis");
        return true;
    }

    tracing::debug!("Analysis file '{}' is up to date", analysis_file);
    false
}

fn check_doctype_files_newer(config: &Config, analysis_mtime: std::time::SystemTime) -> bool {
    use std::fs;
    use std::path::Path;

    let app_path = Path::new(&config.app_absolute_path);
    let modules_txt = app_path.join(&config.app_relative_path).join("modules.txt");

    println!("Checking doctype files in app path: {:?}", app_path);

    // Read modules.txt to get module list
    let modules_content = match fs::read_to_string(&modules_txt) {
        Ok(content) => content,
        Err(_) => return false,
    };

    for line in modules_content.lines() {
        let module_title = line.trim();
        if module_title.is_empty() {
            continue;
        }

        let module_dir = to_snakec(module_title);
        let module_path = app_path.join(&config.app_relative_path).join(&module_dir);

        println!("Checking module: {}", module_title);
        println!("Module path: {:?}", module_path);
        println!("Module dir: {}", module_dir);

        // Check doctype directory
        let doctype_path = module_path.join("doctype");
        // tracing::debug!("Checking doctype path: {:?}", doctype_path);
        if !doctype_path.exists() || !doctype_path.is_dir() {
            continue;
        }

        // Check each doctype directory
        if let Ok(entries) = fs::read_dir(&doctype_path) {
            for entry in entries.flatten() {
                println!("reading entry: {:?}", entry.path());
                if !entry.file_type().map_or(false, |ft| ft.is_dir()) {
                    continue;
                }

                let doctype_name = entry.file_name().to_string_lossy().to_string();
                if doctype_name.is_empty()
                    || ["__pycache__", ".git"].contains(&doctype_name.as_str())
                {
                    continue;
                }

                let doctype_dir = entry.path();

                // Check .py, .js, and .json files
                let files_to_check = vec![
                    doctype_dir.join(format!("{}.py", &doctype_name)),
                    // doctype_dir.join(format!("{}.js", &doctype_name)),
                    doctype_dir.join(format!("{}.json", &doctype_name)),
                ];

                for file_path in files_to_check {
                    if file_path.exists() {
                        if let Ok(metadata) = fs::metadata(&file_path) {
                            if let Ok(mtime) = metadata.modified() {
                                if mtime > analysis_mtime {
                                    tracing::debug!("File {:?} is newer than analysis", file_path);
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
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

    // Auto-run analysis if needed
    let analysis_file = "analyzed_output.dat";
    let should_analyze = should_run_analysis(&config, analysis_file);

    if should_analyze {
        tracing::info!("Running automatic analysis...");
        let app_dir = &config.app_absolute_path;
        let relative_path = format!("{}", config.app_relative_path);

        if let Err(e) = crate::analyze::analyze_frappe_app(app_dir, &relative_path, analysis_file) {
            tracing::error!("Failed to run automatic analysis: {}", e);
        } else {
            tracing::info!("Automatic analysis completed");
        }
    }

    tracing::debug!("Load analyzed data: {}", analysis_file);
    let anal = AnalyzedData::from_file(analysis_file)
        .map(|data| {
            tracing::debug!(
                "Analyzed Data:\n\
                 + {} modules\n\
                 + {} doctypes\n\
                ",
                data.modules.len(),
                data.doctypes.len()
            );
            data
        })
        .unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to load analyzed data from '{}': {}. Using empty analysis.",
                analysis_file,
                e
            );
            AnalyzedData {
                doctypes: Vec::new(),
                modules: Vec::new(),
                symbol_refs: None,
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
        // assert!(r.has_route("get_function_signature"));
        assert!(r.has_route("get_doctype"));
        assert!(r.has_route("create_doctype_template"));
        assert!(r.has_route("create_web_page"));
        assert!(r.has_route("run_tests"));
        assert!(r.has_route("analyze_links"));
        assert!(r.has_route("find_field_usage"));
        assert!(r.has_route("run_bench_command"));
        assert!(r.has_route("bench_execute"));
        assert!(r.has_route("run_db_command"));
        assert!(r.has_route("create_test_template"));
        assert!(r.has_route("list_doctypes"));
    }

    // #[tokio::test]
    // async fn prompt_has_route() {
    //     let r = ProjectExplorer::prompt_router();
    //     assert!(r.has_route("example_prompt"));
    //     let attr = ProjectExplorer::example_prompt_prompt_attr();
    //     assert_eq!(attr.name, "example_prompt");
    // }
}
