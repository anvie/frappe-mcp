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
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;

use anyhow::{bail, Context, Result};

pub fn run_bench_command<I, S>(config: &Config, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let bench_dir = Path::new(&config.frappe_bench_dir);

    // Tentukan folder bin venv
    let venv_dir = bench_dir.join("env");
    let venv_bin = if cfg!(target_os = "windows") {
        venv_dir.join("Scripts")
    } else {
        venv_dir.join("bin")
    };

    // Siapkan PATH baru: venv/bin + PATH lama
    let old_path = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = env::split_paths(&old_path).collect();
    // prepend (di depan)
    paths.insert(0, venv_bin.clone());
    let new_path = env::join_paths(paths).context("join PATH failed")?;

    // Bangun perintah bench; biarkan resolve dari PATH global (/usr/local/bin/bench)
    let mut cmd = Command::new("bench");
    cmd.current_dir(bench_dir)
        .env("PATH", &new_path)
        .env("VIRTUAL_ENV", &venv_dir)
        // opsional supaya pip tidak nulis ke user site
        .env("PIP_USER", "0")
        .arg("--site")
        .arg(&config.site)
        .args(args);

    let output = cmd.output().with_context(|| "Failed to spawn bench")?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        let truncated_stdout = truncate_output(&stdout, 3000);
        let truncated_stderr = truncate_output(&stderr, 3000);
        bail!(format!(
            "bench exited with code {:?}\nSTDOUT:\n{}\n\nSTDERR:\n{}",
            output.status.code(),
            truncated_stdout,
            truncated_stderr
        ));
    }

    let truncated_stdout = truncate_output(&stdout, 5000);
    let truncated_stderr = truncate_output(&stderr, 5000);

    Ok(format!(
        "STDOUT:\n{}\n\nSTDERR:\n{}",
        truncated_stdout, truncated_stderr
    ))
}

pub fn run_db_command(config: &Config, sql: &str) -> Result<String> {
    run_bench_command(config, &["mariadb", "-e", sql])
}

fn truncate_output(output: &str, max_chars: usize) -> String {
    // If within character limit, return as-is
    if output.len() <= max_chars {
        return output.to_string();
    }

    let mut result = String::new();

    // Find the last complete character within the limit
    let mut char_count = 0;
    for ch in output.chars() {
        if char_count + ch.len_utf8() > max_chars {
            break;
        }
        result.push(ch);
        char_count += ch.len_utf8();
    }

    let truncated_chars = output.len() - result.len();
    result.push_str(&format!("\n... (truncated {} chars)", truncated_chars));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_output_within_limit() {
        let input = "Hello world";
        let result = truncate_output(input, 20);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_truncate_output_exact_limit() {
        let input = "Hello world";
        let result = truncate_output(input, 11);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_truncate_output_exceeds_limit() {
        let input = "Hello world this is a long string";
        let result = truncate_output(input, 10);
        assert_eq!(result, "Hello worl\n... (truncated 23 chars)");
    }

    #[test]
    fn test_truncate_output_with_unicode() {
        let input = "Hello 🌍 world";
        let result = truncate_output(input, 10);
        // The emoji takes 4 bytes, so "Hello 🌍" is 9 bytes, can't fit " world"
        assert_eq!(result, "Hello 🌍\n... (truncated 6 chars)");
    }

    #[test]
    fn test_truncate_output_empty_string() {
        let input = "";
        let result = truncate_output(input, 10);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_output_single_char() {
        let input = "a";
        let result = truncate_output(input, 0);
        assert_eq!(result, "\n... (truncated 1 chars)");
    }

    #[test]
    fn test_truncate_output_newlines_preserved() {
        let input = "Line 1\nLine 2\nLine 3";
        let result = truncate_output(input, 10);
        assert_eq!(result, "Line 1\nLin\n... (truncated 10 chars)");
    }
}
