#![allow(dead_code)]
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;

use anyhow::{bail, Context, Result};

// pub fn run_bench_command<I, S>(config: &Config, args: I) -> Result<String>
// where
//     I: IntoIterator<Item = S> + std::fmt::Debug,
//     S: AsRef<OsStr>,
// {
//     let bench_cmd = if cfg!(target_os = "windows") {
//         "bench.bat"
//     } else {
//         "bench"
//     };
//     let bench_dir = &config.frappe_bench_dir;
//
//     tracing::debug!(
//         "Running bench command: {} {:?}\nWORKDIR={}",
//         bench_cmd,
//         args,
//         bench_dir
//     );
//
//     let output = Command::new(bench_cmd)
//         .arg("--site")
//         .arg(&config.site)
//         .args(args)
//         .current_dir(bench_dir)
//         .output();
//
//     match output {
//         Ok(result) => {
//             let stdout = String::from_utf8_lossy(&result.stdout);
//             let stderr = String::from_utf8_lossy(&result.stderr);
//             Ok(format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
//         }
//         Err(e) => {
//             tracing::error!("Failed to execute bench command: {}", e);
//             Err(e.into())
//         }
//     }
// }

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
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let truncated_stdout = truncate_output(&stdout, 50);
        let truncated_stderr = truncate_output(&stderr, 50);
        bail!(format!(
            "bench exited with code {:?}\nSTDOUT:\n{}\n\nSTDERR:\n{}",
            output.status.code(),
            truncated_stdout,
            truncated_stderr
        ));
    }

    let truncated_stdout = truncate_output(&stdout, 50);
    let truncated_stderr = truncate_output(&stderr, 50);
    
    Ok(format!("STDOUT:\n{}\n\nSTDERR:\n{}", truncated_stdout, truncated_stderr))
}

pub fn run_mariadb_command(config: &Config, sql: &str) -> Result<String> {
    run_bench_command(config, &["mariadb", "-e", sql])
}

fn truncate_output(output: &str, max_lines: usize) -> String {
    const MAX_CHARS: usize = 2000;
    
    let lines: Vec<&str> = output.lines().collect();
    
    // If within both limits, return as-is
    if lines.len() <= max_lines && output.len() <= MAX_CHARS {
        return output.to_string();
    }
    
    let mut result = String::new();
    let mut line_count = 0;
    
    for line in lines.iter() {
        if line_count >= max_lines || result.len() + line.len() + 1 > MAX_CHARS {
            break;
        }
        
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
        line_count += 1;
    }
    
    let total_lines = lines.len();
    let total_chars = output.len();
    let truncated_lines = total_lines - line_count;
    let truncated_chars = total_chars - result.len();
    
    if truncated_lines > 0 || truncated_chars > 0 {
        result.push_str(&format!("\n... (truncated {} lines, {} chars)", truncated_lines, truncated_chars));
    }
    
    result
}
