#![allow(dead_code)]
use std::fs;
use std::path::Path;

use crate::analyze::AnalyzedData;
use crate::config::Config;
use crate::stringutil::to_snakec;
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn create_web_page(
    config: &Config,
    _anal: &AnalyzedData,
    path: &str,
    title: Option<String>,
    include_css: Option<bool>,
    include_js: Option<bool>,
) -> McpResult {
    let full_path = format!(
        "{}/{}/www/{}",
        config.app_absolute_path,
        to_snakec(&config.app_name),
        path
    );
    let file_path = Path::new(&full_path);

    // Extract filename without extension for title
    let filename = file_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Web Page");

    let page_title = title.unwrap_or_else(|| filename.to_string());
    let css_enabled = include_css.unwrap_or(true);
    let js_enabled = include_js.unwrap_or(true);

    // Check if file already exists
    if file_path.exists() {
        mcp_return!(format!("File already exists at: {}", full_path));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            mcp_return!(format!(
                "Failed to create directory {}: {}",
                parent.display(),
                e
            ));
        }
    }

    let mut result = Vec::new();

    // Create HTML file
    let html_content =
        create_html_boilerplate(&page_title, css_enabled, js_enabled, &to_snakec(filename));
    if let Err(e) = fs::write(&full_path, html_content) {
        mcp_return!(format!("Failed to write HTML file: {}", e));
    }
    result.push(format!("✓ Created HTML: {}", full_path));

    // Create CSS file if requested
    if css_enabled {
        let css_path = full_path.replace(".html", ".css");
        let css_content = create_css_boilerplate();
        if let Err(e) = fs::write(&css_path, css_content) {
            mcp_return!(format!("Failed to write CSS file: {}", e));
        }
        result.push(format!("✓ Created CSS: {}", css_path));
    }

    // Create JavaScript file if requested
    if js_enabled {
        let js_path = full_path.replace(".html", ".js");
        let js_content = create_js_boilerplate(&to_snakec(filename));
        if let Err(e) = fs::write(&js_path, js_content) {
            mcp_return!(format!("Failed to write JavaScript file: {}", e));
        }
        result.push(format!("✓ Created JavaScript: {}", js_path));
    }

    let summary = format!(
        "Web page '{}' created successfully:\n\n{}\n\nNext steps:\n- Customize the HTML structure as needed\n- Add your own styles to the CSS file\n- Implement interactive features in the JavaScript file",
        page_title,
        result.join("\n")
    );

    mcp_return!(summary)
}

fn create_html_boilerplate(
    title: &str,
    include_css: bool,
    include_js: bool,
    filename: &str,
) -> String {
    let css_link = if include_css {
        format!("    <link rel=\"stylesheet\" href=\"{}.css\">\n", filename)
    } else {
        String::new()
    };

    let js_script = if include_js {
        format!("    <script src=\"{}.js\"></script>\n", filename)
    } else {
        String::new()
    };

    format!(
        r#"{{% extends "templates/web.html" %}}

{{% block title %}}{}{{% endblock %}}

{{% block head_include %}}
<meta name="viewport" content="width=device-width, initial-scale=1.0">


  <meta name="description" content="">
  <meta name="robots" content="index, follow">


<!-- Meta Tags -->
{{% for tag in meta_tags %}}
<meta {{% for key, value in tag.items() %}}{{ key }}="{{ value }}" {{% endfor %}}>
{{% endfor %}}

<!-- Font optimization -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>

{}

{{% endblock %}}


{{% block content %}}
    
    <!-- Main content area -->

{}    <script>
        // Basic page initialization
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('Page loaded: {}');
        }});
    </script>
{{% endblock %}}
"#,
        title, css_link, title, js_script
    )
}

fn create_css_boilerplate() -> String {
    r#"/* Custom styles for your web page */
"#
    .to_string()
}

fn create_js_boilerplate(filename: &str) -> String {
    format!(
        r#"// JavaScript for {} page

/**
 * Page initialization
 */
document.addEventListener('DOMContentLoaded', function() {{
    console.log('Initializing {} page...');
    
    // Initialize page components
    initializeComponents();
    
    // Set up event listeners
    setupEventListeners();
}});

/**
 * Initialize page components
 */
function initializeComponents() {{
    // Add your component initialization logic here
    console.log('Components initialized');
}}
"#,
        filename, filename
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn mock_config() -> Config {
        Config {
            frappe_bench_dir: "/tmp".to_string(),
            app_name: "test_app".to_string(),
            app_absolute_path: "/tmp/test".to_string(),
            app_relative_path: "test_app".to_string(),
            site: "frontend".to_string(),
        }
    }

    #[test]
    fn test_create_html_boilerplate() {
        let html = create_html_boilerplate("Test Page", true, true, "test_page");
        println!("{}", html);
        assert!(html.contains("templates/web.html"));
        assert!(html.contains("{% block title %}Test Page{% endblock %}"));
        assert!(html.contains("test_page.css"));
        assert!(html.contains("test_page.js"));
    }

    #[test]
    fn test_create_html_without_css_js() {
        let html = create_html_boilerplate("Test Page", false, false, "test_page");
        assert!(!html.contains("test_page.css"));
        assert!(!html.contains("test_page.js"));
    }

    #[test]
    fn test_create_css_boilerplate() {
        let css = create_css_boilerplate();
        assert!(css.contains("Custom styles"));
    }

    #[test]
    fn test_create_js_boilerplate() {
        let js = create_js_boilerplate("test_page");
        assert!(js.contains("DOMContentLoaded"));
        assert!(js.contains("test_page"));
        assert!(js.contains("initializeComponents"));
    }
}
