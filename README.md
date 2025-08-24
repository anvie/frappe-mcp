# Frappe MCP

A Model Context Protocol (MCP) server designed to help AI agents understand and work with Frappe applications. This tool provides semantic analysis and exploration capabilities for Frappe codebases.

## Features

- **Frappe App Analysis**: Analyzes Frappe application structure including modules, doctypes, and relationships
- **Function Signature Extraction**: Retrieves function signatures from Python code with optional module filtering
- **DocType Information**: Provides detailed information about Frappe DocTypes including metadata
- **Symbol Search**: Search for symbols across the project codebase (planned feature)
- **MCP Integration**: Standard MCP server interface for AI agent communication

## Tools Available

- `get_function_signature`: Extract function signatures from app source files
- `get_doctype`: Get DocType information by name (e.g., "Sales Invoice")
- `find_symbols`: Search for symbols across project files (in development)
- `echo`: Debug tool for testing JSON parameter passing

## Installation & Usage

### Prerequisites

- Rust (latest stable version)
- Frappe application codebase to analyze

### Setup

1. Clone and build:

```bash
git clone <repository-url>
cd frappe-mcp
cargo build --release
```

2. Analyze your Frappe app:

```bash
cargo run -- --config default.conf analyze --app-dir /path/to/frappe/app --relative-path appname/
```

3. Run the MCP server:

```bash
cargo run
```

### Testing

Use the MCP Inspector to test the server:

```bash
cargo build --release
npx @modelcontextprotocol/inspector ./target/release/frappe_mcp
```

Available test methods: `find_symbols`, `get_function_signature`, `find_doctype`

### Configuration

Create a `default.conf` file to configure the server (see project structure for examples).

## Project Structure

- `src/main.rs`: CLI entry point with analyze and run commands
- `src/server.rs`: MCP server implementation with tool routing
- `src/analyze.rs`: Frappe application analysis logic
- `src/functools/`: Function signature and DocType extraction utilities

**Author**: Robin Syihab
