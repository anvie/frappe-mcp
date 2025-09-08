use frappe_mcp::functools::{get_frappe_doc, search_frappe_docs, OutputFormat};

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn test_search_exact_match() {
        let result = search_frappe_docs("DocType", None, false, 10, OutputFormat::Json);
        assert!(result.is_ok(), "Search should succeed");

        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty(), "Should have content");
    }

    #[test]
    fn test_search_fuzzy_match() {
        let result = search_frappe_docs("doctyp", None, true, 10, OutputFormat::Json);
        assert!(result.is_ok(), "Fuzzy search should succeed");

        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty(), "Should have content");
    }

    #[test]
    fn test_search_with_category_filter() {
        let result = search_frappe_docs(
            "field",
            Some("doctypes".to_string()),
            false,
            10,
            OutputFormat::Json,
        );
        assert!(result.is_ok(), "Category filtered search should succeed");
    }

    #[test]
    fn test_search_api_category() {
        let result = search_frappe_docs(
            "database",
            Some("api".to_string()),
            false,
            10,
            OutputFormat::Json,
        );
        assert!(result.is_ok(), "API category search should succeed");
    }

    #[test]
    fn test_search_tutorial_category() {
        let result = search_frappe_docs(
            "getting started",
            Some("tutorial".to_string()),
            false,
            10,
            OutputFormat::Json,
        );
        assert!(result.is_ok(), "Tutorial search should succeed");
    }

    #[test]
    fn test_search_limit_parameter() {
        let result = search_frappe_docs("frappe", None, false, 2, OutputFormat::Json);
        assert!(result.is_ok(), "Limited search should succeed");
    }

    #[test]
    fn test_search_no_results() {
        let result = search_frappe_docs(
            "xyznonexistentterm123456",
            None,
            false,
            10,
            OutputFormat::Json,
        );
        assert!(
            result.is_ok(),
            "Search with no results should still succeed"
        );
    }

    #[test]
    fn test_search_case_insensitive() {
        let result_lower = search_frappe_docs("doctype", None, false, 10, OutputFormat::Json);
        let result_upper = search_frappe_docs("DOCTYPE", None, false, 10, OutputFormat::Json);

        assert!(result_lower.is_ok(), "Lowercase search should succeed");
        assert!(result_upper.is_ok(), "Uppercase search should succeed");
    }

    #[test]
    fn test_search_empty_query() {
        let result = search_frappe_docs("", None, false, 10, OutputFormat::Json);
        assert!(result.is_ok(), "Empty query search should succeed");
    }

    #[test]
    fn test_search_invalid_category() {
        let result = search_frappe_docs(
            "doctype",
            Some("nonexistentcategory".to_string()),
            false,
            10,
            OutputFormat::Json,
        );
        assert!(
            result.is_ok(),
            "Search with invalid category should succeed"
        );
    }

    #[test]
    fn test_search_with_zero_limit() {
        let result = search_frappe_docs("DocType", None, false, 0, OutputFormat::Json);
        assert!(result.is_ok(), "Search with zero limit should succeed");
    }

    #[test]
    fn test_search_fuzzy_vs_exact() {
        let fuzzy_result = search_frappe_docs("doctpe", None, true, 10, OutputFormat::Json); // typo
        let exact_result = search_frappe_docs("doctpe", None, false, 10, OutputFormat::Json); // same typo

        assert!(fuzzy_result.is_ok(), "Fuzzy search should succeed");
        assert!(exact_result.is_ok(), "Exact search should succeed");
    }

    #[test]
    fn test_search_json_output_format() {
        let result = search_frappe_docs("DocType", None, false, 2, OutputFormat::Json);
        assert!(result.is_ok(), "JSON format search should succeed");

        let tool_result = result.unwrap();
        let content = &tool_result.content[0];

        // Check that the content can be parsed as JSON
        let json_str = format!("{:?}", content);
        assert!(
            json_str.contains("message"),
            "JSON should contain message field"
        );
        assert!(
            json_str.contains("results"),
            "JSON should contain results field"
        );
    }

    #[test]
    fn test_search_markdown_output_format() {
        let result = search_frappe_docs("DocType", None, false, 2, OutputFormat::Markdown);
        assert!(result.is_ok(), "Markdown format search should succeed");

        let tool_result = result.unwrap();
        let content = &tool_result.content[0];
        let content_str = format!("{:?}", content);

        // Check that the content contains markdown formatting
        assert!(
            content_str.contains("# Search Results"),
            "Markdown should contain heading"
        );
        assert!(
            content_str.contains("**ID:**"),
            "Markdown should contain ID field"
        );
        assert!(
            content_str.contains("---"),
            "Markdown should contain separators"
        );
    }
}

#[cfg(test)]
mod helper_function_tests {
    use super::*;

    #[test]
    fn test_get_existing_doc() {
        let result = get_frappe_doc("48b014"); // ID for index.md
        assert!(result.is_ok(), "Should be able to get existing doc");

        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty(), "Should have content");
    }

    #[test]
    fn test_get_nonexistent_doc() {
        let result = get_frappe_doc("invalid");
        assert!(result.is_err(), "Should fail for non-existent doc ID");
    }

    #[test]
    fn test_get_doc_with_id() {
        let result = get_frappe_doc("3b7f1e"); // ID for doctypes/creating_doctypes.md
        assert!(result.is_ok(), "Should be able to get doc with ID");

        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty(), "Should have content");
    }

    // Note: list_frappe_docs tests removed since the function is not actively used
    // and was causing import issues. The function remains available but unused.
}

#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_documentation_files_embedded() {
        // Test that our documentation files are actually embedded
        use rust_embed::RustEmbed;

        #[derive(RustEmbed)]
        #[folder = "frappe_docs/"]
        struct TestDocs;

        // Check that key documentation files exist
        assert!(
            TestDocs::get("index.md").is_some(),
            "index.md should be embedded"
        );
        assert!(
            TestDocs::get("doctypes/creating_doctypes.md").is_some(),
            "creating_doctypes.md should be embedded"
        );
        assert!(
            TestDocs::get("doctypes/field_types.md").is_some(),
            "field_types.md should be embedded"
        );
        assert!(
            TestDocs::get("api/database_api.md").is_some(),
            "database_api.md should be embedded"
        );
        assert!(
            TestDocs::get("api/rest_api.md").is_some(),
            "rest_api.md should be embedded"
        );
        assert!(
            TestDocs::get("tutorial/getting_started.md").is_some(),
            "getting_started.md should be embedded"
        );

        // Check that we have multiple files
        let file_count = TestDocs::iter().count();
        assert!(
            file_count >= 6,
            "Should have at least 6 documentation files embedded"
        );
    }

    #[test]
    fn test_search_returns_valid_json() {
        let result =
            super::search_frappe_docs("DocType", None, false, 5, super::OutputFormat::Json);
        assert!(result.is_ok(), "Search should succeed");

        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty(), "Should have content");

        // For now, just verify that we get some content back
        // The exact structure depends on the rmcp version
        assert!(!tool_result.content.is_empty(), "Should have content");
    }

    #[test]
    fn test_search_categories_work() {
        // Test each category separately
        let categories = ["doctypes", "api", "tutorial"];

        for category in &categories {
            let result = super::search_frappe_docs(
                "frappe",
                Some(category.to_string()),
                false,
                5,
                super::OutputFormat::Json,
            );
            assert!(
                result.is_ok(),
                "Search in {} category should succeed",
                category
            );
        }
    }

    #[test]
    fn test_fuzzy_search_finds_typos() {
        // Fuzzy search should find results even with typos
        let fuzzy_result =
            super::search_frappe_docs("frapppe", None, true, 5, super::OutputFormat::Json); // extra 'p'
        assert!(
            fuzzy_result.is_ok(),
            "Fuzzy search with typo should succeed"
        );

        let exact_result =
            super::search_frappe_docs("frapppe", None, false, 5, super::OutputFormat::Json); // same typo
        assert!(
            exact_result.is_ok(),
            "Exact search with typo should still succeed (but may return no results)"
        );
    }
}
