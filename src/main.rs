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

#[derive(Parser, Debug)]
#[command(name = "frappe-mcp")]
#[command(about = "MCP server for helping Agentic AI coding in Frappe environment")]
#[command(author, version, long_about=None)]
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

    return (args, config);
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
        CommandEnum::Version => {
            println!("Version 0.0.1");
            return;
        }
    }

    let _ = server::run(config).await;
}
