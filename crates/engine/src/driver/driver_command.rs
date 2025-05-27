use anyhow::Result;
use florca_core::driver::DriverArgs;
use florca_core::run::{RunId, RunRequest};
use std::{path::Path, process::Stdio};
use tokio::process::Child;

pub fn spawn_driver(
    run_request: RunRequest,
    deployment_path: &Path,
    outfile_path: &Path,
    run_id: RunId,
    deno_lock_path: &Path,
) -> Result<Child> {
    let args = build_args(
        run_request,
        deployment_path,
        outfile_path,
        run_id,
        deno_lock_path,
    )?;
    let child = tokio::process::Command::new("deno")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("DENO_TLS_CA_STORE", "system")
        .spawn()?;
    Ok(child)
}

fn build_args(
    run_request: RunRequest,
    deployment_path: &Path,
    outfile_path: &Path,
    run_id: RunId,
    deno_lock_path: &Path,
) -> Result<Vec<String>> {
    let run_args = DriverArgs {
        run_id,
        deployment_name: run_request.deployment_name,
        deployment_path: deployment_path.to_path_buf(),
        entry_point: run_request.entry_point,
        input: run_request.input,
        params: run_request.params,
        outfile_path: outfile_path.to_path_buf(),
    };
    let args = vec![
        "run".to_string(),
        format!("--lock={}", deno_lock_path.to_string_lossy()),
        "--allow-all".to_string(),
        "--unstable-ffi".to_string(),
        "packages/driver/main.ts".to_string(),
        serde_json::to_string(&run_args)?,
    ];
    Ok(args)
}
