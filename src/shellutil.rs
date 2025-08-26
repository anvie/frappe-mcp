#![allow(dead_code)]
use std::ffi::OsStr;
use std::process::Command;

use crate::config::Config;

use anyhow::Result;

pub fn run_bench_command<I, S>(config: &Config, args: I) -> Result<String>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    let bench_cmd = if cfg!(target_os = "windows") {
        "bench.bat"
    } else {
        "bench"
    };
    let bench_dir = &config.frappe_bench_dir;

    tracing::debug!(
        "Running bench command: {} {:?}\nWORKDIR={}",
        bench_cmd,
        args,
        bench_dir
    );

    let output = Command::new(bench_cmd)
        .arg("--site")
        .arg(&config.site)
        .args(args)
        .current_dir(bench_dir)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            Ok(format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
        }
        Err(e) => {
            tracing::error!("Failed to execute bench command: {}", e);
            Err(e.into())
        }
    }
}

pub fn run_mariadb_command(config: &Config, sql: &str) -> Result<String> {
    run_bench_command(config, &["mariadb", "-e", sql])
}
