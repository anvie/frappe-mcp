# Frappe MCP

A Model Context Protocol (MCP) server designed to help AI agents understand and work with Frappe applications. This tool provides semantic analysis and exploration capabilities for Frappe codebases.

## Features

- **Automatic Analysis**: Automatically analyzes Frappe application structure on server startup when files change
- **DocType Management**: Complete DocType lifecycle including information retrieval, template generation, and database schema access
- **Code Analysis**: Symbol search, function signature extraction, and field usage tracking
- **Testing Integration**: Execute Frappe unit tests for modules or specific DocTypes
- **Development Tools**: Generate boilerplate code for DocTypes and web pages
- **Database Access**: Direct MariaDB query execution and bench command integration
- **Relationship Mapping**: Analyze and visualize DocType relationships and dependencies
- **MCP Integration**: Standard MCP server interface for AI agent communication

## Tools Available

### Core Analysis Tools

- **`find_symbols`**: Search for symbols across the app source files with fuzzy matching support
- **`get_function_signature`**: Extract function signatures from app source files, optionally within specific modules
- **`find_field_usage`**: Search for references to specific DocType fields in code

### DocType Management

- **`get_doctype`**: Get comprehensive DocType information by name (e.g., "Sales Invoice")
- **`get_doctype_db_schema`**: Get the database schema for a specific DocType
- **`create_doctype_template`**: Generate boilerplate DocType structure with JSON metadata, Python controller, and JS form files
- **`analyze_links`**: Analyze and map relationships between DocTypes by examining Link, Table, and Select fields

### Development & Testing

- **`create_web_page`**: Generate boilerplate web page files with HTML, CSS, and JavaScript structure
- **`run_tests`**: Execute unit tests for specific modules, DocTypes, or entire app using bench run-tests

### System Integration

- **`run_bench_command`**: Run arbitrary bench command with arguments (e.g., migrate, install-app)
- **`run_db_command`**: Execute SQL queries via bench mariadb command

## Installation & Usage

### Prerequisites

- Rust (latest stable version)
- Frappe application codebase to analyze

### Setup

1. Clone and build:

```bash
git clone https://github.com/anvie/frappe-mcp.git
cd frappe-mcp
cargo build --release
```

2. Configure your Frappe app by creating a `frappe-mcp.conf` file (see Configuration section below)

3. Run the MCP server:

```bash
cargo run -- --config frappe-mcp.conf run
```

The server will automatically analyze your Frappe application on first run or when source files change.

### Testing

Use the MCP Inspector to test the server:

```bash
cargo build --release
npx @modelcontextprotocol/inspector -- ./target/release/frappe_mcp --config frappe-mcp.conf run
```

Available test methods include all tools listed above: `find_symbols`, `get_function_signature`, `get_doctype`, `create_doctype_template`, `run_tests`, `analyze_links`, `create_web_page`, `find_field_usage`, `run_bench_command`, `get_doctype_db_schema`, `run_db_command`

### Configuration

Create a `frappe-mcp.conf` file to configure the server. Example configuration:

```toml
frappe_bench_dir = "/path/to/frappe-bench"
app_relative_path = "your-app-name"
app_name = "Your App Name"
site = "yoursite.localhost"
```

Configuration parameters:

- `frappe_bench_dir`: Path to your Frappe bench directory
- `app_relative_path`: Name of your app directory within the bench/apps folder
- `app_name`: Display name of your app
- `site`: Frappe site name (defaults to "frontend" if not specified)

### Manual Analysis (Optional)

While the server automatically analyzes your app on startup, you can also run analysis manually:

```bash
cargo run -- --config frappe-mcp.conf analyze --app-dir /path/to/frappe-bench/apps/your-app
```

This generates an `analyzed_output.dat` file with structured information about your app's modules and DocTypes.

## Project Structure

- `src/main.rs`: CLI entry point with analyze and run commands
- `src/server.rs`: MCP server implementation with tool routing
- `src/analyze.rs`: Frappe application analysis logic
- `src/functools/`: Function signature and DocType extraction utilities

**Author**: Robin Syihab
