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
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::{
    analyze::AnalyzedData,
    stringutil::{to_kebabc, to_snakec},
};
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn create_web_page(
    config: &Config,
    _anal: &AnalyzedData,
    slug: &str,
    title: Option<String>,
    include_css: Option<bool>,
    include_js: Option<bool>,
) -> McpResult {
    let base_dir = format!(
        "{}/{}/www/{}",
        config.app_absolute_path,
        to_snakec(&config.app_name),
        slug
    );
    let base_dir = Path::new(&base_dir);
    let index_html = base_dir.join("index.html");

    // Extract filename without extension for title if title is not provided.
    let page_title = title.unwrap_or_else(|| slug.to_string());
    let css_enabled = include_css.unwrap_or(true);
    let js_enabled = include_js.unwrap_or(true);

    // Check if file already exists
    if index_html.exists() {
        mcp_return!(format!("File already exists at: {}", index_html.display()));
    }

    // Create parent directories if they don't exist
    if !base_dir.exists() {
        if let Err(e) = fs::create_dir_all(base_dir) {
            mcp_return!(format!(
                "Failed to create directory {}: {}",
                base_dir.display(),
                e
            ));
        }
    }

    let filename = slug.split('/').last().unwrap_or("index").to_string();

    let mut result = Vec::new();

    // Create HTML file
    let html_content =
        create_html_boilerplate(&page_title, css_enabled, js_enabled, &to_kebabc(&filename));
    if let Err(e) = fs::write(&index_html, html_content) {
        mcp_return!(format!("Failed to write HTML file: {}", e));
    }
    result.push(format!("✓ Created HTML: {}", index_html.display()));

    // Create CSS file if requested
    if css_enabled {
        let css_path = base_dir.join(format!("{}.css", filename));
        let css_content = create_css_boilerplate(&page_title);
        if let Err(e) = fs::write(&css_path, css_content) {
            mcp_return!(format!("Failed to write CSS file: {}", e));
        }
        result.push(format!("✓ Created CSS: {}", css_path.display()));
    }

    // Create JavaScript file if requested
    if js_enabled {
        let js_path = base_dir.join(format!("{}.js", filename));
        let js_content = create_js_boilerplate(&page_title);
        if let Err(e) = fs::write(&js_path, js_content) {
            mcp_return!(format!("Failed to write JavaScript file: {}", e));
        }
        result.push(format!("✓ Created JavaScript: {}", js_path.display()));
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
        title, css_link, js_script, title
    )
}

fn create_css_boilerplate(title: &str) -> String {
    format!(
        r#"/* Custom styles for {} page */
"#,
        title
    )
}

fn create_js_boilerplate(title: &str) -> String {
    format!(
        r#"// JavaScript for {} page

/**
 * Page initialization
 */
$(document).ready(function () {{
  console.log("Initializing {} page...");

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
        title, title
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::AnalyzedData;
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
        let css = create_css_boilerplate("test");
        assert!(css.contains("Custom styles"));
        assert!(css.contains("test page"));
    }

    #[test]
    fn test_create_js_boilerplate() {
        let js = create_js_boilerplate("test_page");
        assert!(js.contains("$(document).ready"));
        assert!(js.contains("test_page"));
        assert!(js.contains("initializeComponents"));
    }

    #[test]
    fn test_create_web_page() {
        use std::fs;
        use std::path::Path;

        // Create a temporary test directory in tests/sandbox
        let test_dir = "/tmp/frappe_mcp_test_web_page";
        let app_path = format!("{}/test_app", test_dir);

        // Clean up any existing test directory
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        let config = Config {
            frappe_bench_dir: test_dir.to_string(),
            app_name: "Test App".to_string(),
            app_absolute_path: app_path.clone(),
            app_relative_path: "test_app".to_string(),
            site: "frontend".to_string(),
        };

        // Create a minimal AnalyzedData instance
        let anal = AnalyzedData {
            doctypes: vec![],
            modules: vec![],
            symbol_refs: None,
        };

        // Test 1: Create web page with CSS and JS
        let result = create_web_page(
            &config,
            &anal,
            "about",
            Some("About Us".to_string()),
            Some(true),
            Some(true),
        );
        assert!(result.is_ok());

        // Verify files were created
        // The create_web_page function uses to_snakec on the app_name
        let about_dir = Path::new(&app_path).join("test_app/www/about");
        assert!(about_dir.exists());
        assert!(about_dir.join("index.html").exists());
        assert!(about_dir.join("about.css").exists());
        assert!(about_dir.join("about.js").exists());

        // Verify HTML content
        let html_content = fs::read_to_string(about_dir.join("index.html")).unwrap();
        assert!(html_content.contains("{% block title %}About Us{% endblock %}"));
        assert!(html_content.contains("about.css"));
        assert!(html_content.contains("about.js"));

        // Test 2: Create web page without CSS and JS
        let result = create_web_page(&config, &anal, "contact", None, Some(false), Some(false));
        assert!(result.is_ok());

        let contact_dir = Path::new(&app_path).join("test_app/www/contact");
        assert!(contact_dir.join("index.html").exists());
        assert!(!contact_dir.join("contact.css").exists());
        assert!(!contact_dir.join("contact.js").exists());

        // Test 3: Try to create duplicate page
        let result = create_web_page(&config, &anal, "about", None, None, None);
        assert!(result.is_ok());
        if let Ok(tool_result) = result {
            if let Some(first_content) = tool_result.content.first() {
                if let RawContent::Text(text_content) = &first_content.raw {
                    assert!(text_content.text.contains("File already exists"));
                }
            }
        }

        // Test 4: Create nested page
        let result = create_web_page(
            &config,
            &anal,
            "products/electronics",
            Some("Electronics".to_string()),
            None,
            None,
        );
        assert!(result.is_ok());

        let nested_dir = Path::new(&app_path).join("test_app/www/products/electronics");
        assert!(nested_dir.exists());
        assert!(nested_dir.join("index.html").exists());
        assert!(nested_dir.join("electronics.css").exists());
        assert!(nested_dir.join("electronics.js").exists());

        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
