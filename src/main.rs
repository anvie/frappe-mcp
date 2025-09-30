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
use clap::{Parser, Subcommand};
use std::process::exit;

#[macro_use]
mod macros;
mod analyze;
mod config;
mod fileutil;
mod functools;
mod refs_finder;
mod serdeutil;
mod server;
mod shellutil;
mod stringutil;

use config::Config;
use rmcp::model::{CallToolResult, RawTextContent, ErrorCode};

fn print_tool_result(result: CallToolResult) {
    // For CLI output, extract text from content items
    for content in result.content.iter() {
        if let Some(RawTextContent { text, .. }) = content.as_text() {
            println!("{}", text);
            continue;
        }
        // Extract the raw text content
        let text = format!("{:?}", content);
        // Look for the text field in the debug output
        if let Some(start) = text.find("text: \"") {
            let start = start + 7; // Skip "text: \""
            if let Some(end) = text[start..].find("\" }") {
                let extracted = &text[start..start + end];
                // Unescape the JSON string
                let unescaped = extracted.replace("\\n", "\n").replace("\\\"", "\"");
                println!("{}", unescaped);
                return;
            }
        }
        // Fallback to debug output
        println!("{}", text);
    }
}

#[derive(Parser, Debug)]
#[command(name = "frappe-mcp")]
#[command(about = "Frappe MCP server for helping Agentic AI coding in Frappe environment")]
#[command(author, version = env!("CARGO_PKG_VERSION"), long_about=None)]
struct Args {
    #[arg(short, long, default_value = "frappe-mcp.conf")]
    config: String,

    #[command(subcommand)]
    command: CommandEnum,
}

/// Enum of subcommands
#[derive(Subcommand, Debug)]
enum CommandEnum {
    /// Analyze the codebase and output a analyzed_output.dat file.
    Analyze {
        #[arg(
            short,
            long,
            default_value = "",
            help = "Directory/codebase to analyze"
        )]
        app_dir: String,
        // #[arg(short, long, help = "relative path from root")]
        // relative_path: String,
    },
    /// Run the MCP server
    Run,
    /// Search Frappe documentation
    SearchDocs {
        #[arg(help = "Search query")]
        query: String,
        #[arg(short, long, help = "Filter by category (doctypes, api, tutorial)")]
        category: Option<String>,
        #[arg(short, long, help = "Use fuzzy search", default_value_t = true)]
        fuzzy: bool,
        #[arg(short, long, help = "Maximum number of results", default_value_t = 10)]
        limit: usize,
        #[arg(long, help = "Output format: json or markdown", default_value = "json")]
        format: String,
    },
    /// Read a specific Frappe documentation file
    ReadDoc {
        #[arg(help = "Document ID (e.g., a7b9c3, d8f2e1). Use search-docs to find IDs.")]
        id: String,
    },
    /// Execute functool functions for testing
    Functool {
        #[arg(help = "Function name: get-doctype, list-doctypes, run-bench-command, find-field-usage, find-symbols")]
        function: String,
        #[arg(help = "Function arguments (use functool <function> --help for details)", num_args = 0..)]
        args: Vec<String>,
    },
    /// Print version info
    Version,
}

fn parse_args() -> (Args, Config) {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let config: Config = Config::from_file(&args.config).unwrap_or_else(|err| {
        eprintln!("Error reading config file {}: {}", args.config, err);
        exit(1);
    });

    (args, config)
}

async fn execute_functool(config: &Config, function: &str, args: &[String]) -> Result<CallToolResult, rmcp::ErrorData> {
    // Use default analysis file path
    let analysis_file = "analyzed_output.dat";
    let analyzed_data = analyze::AnalyzedData::from_file(analysis_file).map_err(|_| {
        rmcp::ErrorData::new(
            ErrorCode::INVALID_REQUEST,
            "Failed to load analyzed data. Run 'frappe-mcp analyze' first.",
            None,
        )
    })?;

    match function {
        "get-doctype" | "get_doctype" => {
            if args.is_empty() {
                return Err(rmcp::ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    "get-doctype requires a doctype name",
                    None,
                ));
            }
            let json_only = args.get(1).map(|s| s == "true" || s == "json").unwrap_or(false);
            functools::get_doctype(config, &analyzed_data, &args[0], json_only)
        }
        "list-doctypes" | "list_doctypes" => {
            let module_filter = args.get(0).cloned();
            functools::list_doctypes(config, &analyzed_data, module_filter)
        }
        "run-bench-command" | "run_bench_command" => {
            if args.is_empty() {
                return Err(rmcp::ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    "run-bench-command requires a command",
                    None,
                ));
            }
            // Convert Vec<String> to Vec<&str>
            let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            functools::run_bench_command(config, &analyzed_data, &str_args)
        }
        "find-field-usage" | "find_field_usage" => {
            if args.len() < 2 {
                return Err(rmcp::ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    "find-field-usage requires doctype and field name",
                    None,
                ));
            }
            let limit = args.get(2)
                .and_then(|s| s.parse::<usize>().ok());
            functools::find_field_usage(config, &analyzed_data, &args[0], &args[1], limit)
        }
        "find-symbols" | "find_symbols" => {
            if args.is_empty() {
                return Err(rmcp::ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    "find-symbols requires a symbol pattern",
                    None,
                ));
            }
            let search_in = args.get(1).cloned();
            let fuzzy = args.get(2).map(|s| s == "true").or(Some(false));
            let limit = args.get(3).and_then(|s| s.parse::<usize>().ok());
            functools::find_symbols(config, &analyzed_data, &args[0], search_in, fuzzy, limit)
        }
        _ => {
            Err(rmcp::ErrorData::new(
                ErrorCode::INVALID_REQUEST,
                "Unknown function. Use --help to see available functions.",
                None,
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    let (args, config) = parse_args();

    match args.command {
        CommandEnum::Analyze { app_dir } => {
            // Perform analysis and output to the specified file
            let output = "analyzed_output.dat";
            let relative_path = config.app_relative_path.to_string();
            if let Err(e) = analyze::analyze_frappe_app(&app_dir, &relative_path, output) {
                eprintln!("Analysis error: {}", e);
                exit(1);
            }
            println!("Analysis completed. Output written to {}", output);
            exit(1);
        }
        CommandEnum::Run => {}
        CommandEnum::SearchDocs {
            query,
            category,
            fuzzy,
            limit,
            format,
        } => {
            let output_format = match format.as_str() {
                "json" => functools::OutputFormat::Json,
                "markdown" => functools::OutputFormat::Markdown,
                _ => {
                    eprintln!("Invalid format '{}'. Use 'json' or 'markdown'.", format);
                    exit(1);
                }
            };

            match functools::search_frappe_docs(&query, category, fuzzy, limit, output_format) {
                Ok(result) => {
                    print_tool_result(result);
                }
                Err(e) => {
                    eprintln!("Search error: {}", e.message);
                    exit(1);
                }
            }
            return;
        }
        CommandEnum::ReadDoc { id } => {
            match functools::get_frappe_doc(&id) {
                Ok(result) => {
                    print_tool_result(result);
                }
                Err(e) => {
                    eprintln!("Read error: {}", e.message);
                    exit(1);
                }
            }
            return;
        }
        CommandEnum::Functool { function, args } => {
            match execute_functool(&config, &function, &args).await {
                Ok(result) => {
                    print_tool_result(result);
                }
                Err(e) => {
                    eprintln!("Functool error: {}", e.message);
                    exit(1);
                }
            }
            return;
        }
        CommandEnum::Version => {
            println!("Version {}", env!("CARGO_PKG_VERSION"));
            return;
        }
    }

    let _ = server::run(config).await;
}
