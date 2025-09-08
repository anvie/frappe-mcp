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
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rmcp::{model::*, ErrorData as McpError};
use rust_embed::RustEmbed;
use serde_json::json;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "frappe_docs/"]
struct FrappeDocs;

#[derive(Debug)]
struct DocEntry {
    path: String,
    title: String,
    content: String,
    category: String,
}

pub fn search_frappe_docs(
    query: &str,
    category: Option<String>,
    fuzzy: bool,
    limit: usize,
) -> Result<CallToolResult, McpError> {
    let mut docs = Vec::new();
    
    // Load all embedded documents
    for file in FrappeDocs::iter() {
        let path = file.to_string();
        
        // Skip non-markdown files
        if !path.ends_with(".md") {
            continue;
        }
        
        // Get file content
        if let Some(content_data) = FrappeDocs::get(&path) {
            let content = std::str::from_utf8(content_data.data.as_ref())
                .map_err(|e| McpError {
                    code: rmcp::model::ErrorCode(-32603),
                    message: format!("Failed to read document: {}", e).into(),
                    data: None,
                })?;
            
            // Extract title from first H1 or filename
            let title = extract_title(content, &path);
            
            // Extract category from path
            let doc_category = extract_category(&path);
            
            // Filter by category if specified
            if let Some(ref cat) = category {
                if !doc_category.eq_ignore_ascii_case(cat) {
                    continue;
                }
            }
            
            docs.push(DocEntry {
                path: path.clone(),
                title,
                content: content.to_string(),
                category: doc_category,
            });
        }
    }
    
    // Search through documents
    let mut results = Vec::new();
    
    if fuzzy {
        // Fuzzy search using SkimMatcherV2
        let matcher = SkimMatcherV2::default();
        let mut scored_results: Vec<(i64, &DocEntry)> = Vec::new();
        
        for doc in &docs {
            let mut max_score = 0i64;
            
            // Score against title
            if let Some(score) = matcher.fuzzy_match(&doc.title, query) {
                max_score = max_score.max(score * 2); // Boost title matches
            }
            
            // Score against content
            if let Some(score) = matcher.fuzzy_match(&doc.content, query) {
                max_score = max_score.max(score);
            }
            
            if max_score > 0 {
                scored_results.push((max_score, doc));
            }
        }
        
        // Sort by score (highest first)
        scored_results.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Take top results
        for (score, doc) in scored_results.iter().take(limit) {
            let snippet = extract_snippet(&doc.content, query, 150);
            results.push(json!({
                "path": doc.path,
                "title": doc.title,
                "category": doc.category,
                "score": score,
                "snippet": snippet,
            }));
        }
    } else {
        // Exact search (case-insensitive)
        let query_lower = query.to_lowercase();
        
        for doc in &docs {
            let title_lower = doc.title.to_lowercase();
            let content_lower = doc.content.to_lowercase();
            
            if title_lower.contains(&query_lower) || content_lower.contains(&query_lower) {
                let snippet = extract_snippet(&doc.content, query, 150);
                results.push(json!({
                    "path": doc.path,
                    "title": doc.title,
                    "category": doc.category,
                    "snippet": snippet,
                }));
                
                if results.len() >= limit {
                    break;
                }
            }
        }
    }
    
    // Build response
    let response = if results.is_empty() {
        json!({
            "message": format!("No documentation found for query: '{}'", query),
            "results": [],
            "total": 0
        })
    } else {
        json!({
            "message": format!("Found {} result(s) for query: '{}'", results.len(), query),
            "results": results,
            "total": results.len()
        })
    };
    
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string())
    )]))
}

fn extract_title(content: &str, path: &str) -> String {
    // Try to find first H1 heading
    for line in content.lines() {
        if line.starts_with("# ") {
            return line.trim_start_matches('#').trim().to_string();
        }
    }
    
    // Fallback to filename without extension
    path.split('/')
        .last()
        .unwrap_or(path)
        .trim_end_matches(".md")
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_category(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 1 {
        // Get the directory name
        parts[0].to_string()
    } else {
        "general".to_string()
    }
}

fn extract_snippet(content: &str, query: &str, max_length: usize) -> String {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();
    
    // Find the position of the query in the content
    if let Some(pos) = content_lower.find(&query_lower) {
        // Calculate snippet boundaries
        let start = pos.saturating_sub(50);
        let end = (pos + query.len() + 100).min(content.len());
        
        // Extract snippet
        let snippet = &content[start..end];
        
        // Clean up snippet
        let mut result = snippet.trim().to_string();
        
        // Add ellipsis if truncated
        if start > 0 {
            result = format!("...{}", result);
        }
        if end < content.len() {
            result = format!("{}...", result);
        }
        
        // Remove markdown formatting for readability
        result = result
            .replace("###", "")
            .replace("##", "")
            .replace("#", "")
            .replace("**", "")
            .replace("```", "")
            .replace("\n", " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        // Truncate if still too long
        if result.len() > max_length {
            result.truncate(max_length);
            result.push_str("...");
        }
        
        result
    } else {
        // If query not found, return first part of content
        let mut snippet = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        
        if snippet.len() > max_length {
            snippet.truncate(max_length);
            snippet.push_str("...");
        }
        
        snippet
    }
}

pub fn get_frappe_doc(path: &str) -> Result<CallToolResult, McpError> {
    // Get specific document by path
    if let Some(content_data) = FrappeDocs::get(path) {
        let content = std::str::from_utf8(content_data.data.as_ref())
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to read document: {}", e).into(),
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(content.to_string())]))
    } else {
        Err(McpError {
            code: rmcp::model::ErrorCode(-32602),
            message: format!("Document not found: {}", path).into(),
            data: None,
        })
    }
}

pub fn list_frappe_docs(category: Option<String>) -> Result<CallToolResult, McpError> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    
    for file in FrappeDocs::iter() {
        let path = file.to_string();
        
        if !path.ends_with(".md") {
            continue;
        }
        
        let doc_category = extract_category(&path);
        
        // Filter by category if specified
        if let Some(ref cat) = category {
            if !doc_category.eq_ignore_ascii_case(cat) {
                continue;
            }
        }
        
        categories
            .entry(doc_category)
            .or_insert_with(Vec::new)
            .push(path);
    }
    
    let response = json!({
        "categories": categories,
        "total": categories.values().map(|v| v.len()).sum::<usize>()
    });
    
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string())
    )]))
}