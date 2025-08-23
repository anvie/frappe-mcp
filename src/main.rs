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
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, Write};
use std::{fs, io::ErrorKind, process::exit};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

mod analyze;

/// A basic MCP request
#[derive(Debug, Deserialize)]
struct McpRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// A basic MCP response
#[derive(Debug, Serialize)]
struct McpResponse {
    pub id: Value,
    pub result: Value,
}

/// Helper: wrap text in MCP `content` format
fn wrap_text(text: String) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

/// Mock implementation: find_references
fn find_references(params: &Value) -> Value {
    let symbol = params["symbol"].as_str().unwrap_or("unknown");
    let text = format!(
        "References for '{}':\n- src/main.rs:12\n- lib/utils.rs:33",
        symbol
    );
    wrap_text(text)
}

/// Mock implementation: get_function_signature
fn get_function_signature(params: &Value) -> Value {
    let func = params["name"].as_str().unwrap_or("unknown_func");
    let text = format!(
        "Signature of '{}': fn {}(arg1: i32, arg2: String) -> Result<(), Error>",
        func, func
    );
    wrap_text(text)
}

/// Mock implementation: find_doctype
fn find_doctype(params: &Value) -> Value {
    eprintln!("find_doctype params: {:?}", params);
    let doctype = params["name"].as_str().unwrap_or("DocType");
    let text = format!(
        "Doctype '{}': located at models/{}.rs with fields: id (i32), title (String), created_at (DateTime)",
        doctype, doctype
    );
    wrap_text(text)
}

/// Dispatch functions
fn handle_method(req: &McpRequest) -> Value {
    match req.method.as_str() {
        "find_references" => find_references(req.params.as_ref().unwrap_or(&json!({}))),
        "get_function_signature" => {
            get_function_signature(req.params.as_ref().unwrap_or(&json!({})))
        }
        "find_doctype" => find_doctype(req.params.as_ref().unwrap_or(&json!({}))),
        _ => wrap_text(format!("Unknown method: {}", req.method)),
    }
}

#[tokio::main]
async fn main() {
    use tokio::io::AsyncReadExt;

    let args = parse_args();

    if !args.analyze.is_empty() {
        // Perform analysis and output to the specified file
        let output = "analyzed_output.toml";
        if let Err(e) = analyze::analyze_frappe_app(&args.analyze, &args.relative_path, output) {
            eprintln!("Analysis error: {}", e);
            exit(1);
        }
        println!("Analysis completed. Output written to {}", output);
        exit(0);
    }

    let mut stdin = tokio::io::stdin();
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Writer task
    tokio::spawn(async move {
        let mut stdout = io::stdout();
        while let Some(msg) = rx.recv().await {
            let header = format!("Content-Length: {}\r\n\r\n", msg.len());
            stdout.write_all(header.as_bytes()).unwrap();
            stdout.write_all(msg.as_bytes()).unwrap();
            stdout.flush().unwrap();
        }
    });

    let mut buffer = String::new();

    loop {
        // Read headers
        buffer.clear();
        let mut header_buf = [0u8; 1];
        let mut header_str = String::new();
        while stdin.read_exact(&mut header_buf).await.is_ok() {
            header_str.push(header_buf[0] as char);
            if header_str.ends_with("\r\n\r\n") {
                break;
            }
        }

        // Parse Content-Length
        let mut content_length: Option<usize> = None;
        for line in header_str.split("\r\n") {
            if line.starts_with("Content-Length:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    content_length = parts[1].trim().parse().ok();
                }
            }
        }

        let len = if let Some(l) = content_length {
            l
        } else {
            continue;
        };

        // Read body of exactly `len` bytes
        let mut body_buf = vec![0u8; len];
        stdin.read_exact(&mut body_buf).await.unwrap();
        let body = String::from_utf8_lossy(&body_buf);

        // Parse JSON
        let parsed: serde_json::Result<McpRequest> = serde_json::from_str(&body);
        if let Ok(req) = parsed {
            if let Some(id) = req.id.clone() {
                let result = handle_method(&req);
                let resp = McpResponse { id, result };
                let msg = serde_json::to_string(&resp).unwrap();
                let _ = tx.send(msg).await;
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "frappe-mcp")]
#[command(about = "MCP server for helping Agentic AI coding in Frappe environment")]
#[command(author, version, long_about=None)]
struct Args {
    #[arg(short, long, default_value = "default.conf")]
    config: String,

    #[arg(
        short,
        long,
        default_value = "",
        help = "analyze the codebase and output a toml file"
    )]
    analyze: String,

    #[arg(
        short,
        long,
        default_value = "appname",
        help = "relative path from root"
    )]
    relative_path: String,
}
#[derive(Deserialize, Debug)]
struct Config {
    param: u32,
}

fn parse_args() -> Args {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let config: Config = match fs::read_to_string(&args.config) {
        Ok(config) => toml::from_str(&config).unwrap(),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("`{}` not exists.", args.config);
                exit(2);
            } else {
                panic!("Error: {}", e);
            }
        }
    };

    return args;
}
