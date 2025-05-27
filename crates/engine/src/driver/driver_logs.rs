use anyhow::Result;
use florca_core::{function::FunctionName, invocation::InvocationId, run::RunId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{ChildStderr, ChildStdout},
};
use tracing::{debug, error, info, warn};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[derive(TS)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct PluginLogEvent {
    pub level: LogLevel,
    pub message: String,
    pub data: Option<Value>,
    pub invocation_id: InvocationId,
    pub function_name: FunctionName,
}

pub fn parse_stdout_and_stderr(stdout: ChildStdout, stderr: ChildStderr, run_id: RunId) {
    let stdout_reader = BufReader::new(stdout).lines();
    tokio::spawn(async move {
        read_stdout(stdout_reader, run_id).await.unwrap();
    });

    let stderr_reader = BufReader::new(stderr).lines();
    tokio::spawn(async move {
        read_stderr(stderr_reader, run_id).await.unwrap();
    });
}

async fn read_stdout(
    mut stdout_reader: Lines<BufReader<ChildStdout>>,
    run_id: RunId,
) -> Result<()> {
    while let Some(line) = stdout_reader.next_line().await? {
        if let Ok(log_event) = serde_json::from_str::<PluginLogEvent>(&line) {
            let message = log_event.message;
            let data = log_event
                .data
                .map(|d| serde_json::to_string_pretty(&d).unwrap())
                .map(|s| " ".to_string() + &s)
                .unwrap_or_default();
            match log_event.level {
                LogLevel::Debug => {
                    debug!(target: "driver", run = run_id.to_string(), invocation = log_event.invocation_id.to_string(), function = log_event.function_name.to_string(), "{}{}", message, data);
                }
                LogLevel::Info => {
                    info!(target: "driver", run = run_id.to_string(), invocation = log_event.invocation_id.to_string(), function = log_event.function_name.to_string(), "{}{}", message, data);
                }
                LogLevel::Warn => {
                    warn!(target: "driver", run = run_id.to_string(), invocation = log_event.invocation_id.to_string(), function = log_event.function_name.to_string(), "{}{}", message, data);
                }
                LogLevel::Error => {
                    error!(target: "driver", run = run_id.to_string(), invocation = log_event.invocation_id.to_string(), function = log_event.function_name.to_string(), "{}{}", message, data);
                }
            }
        } else if let Ok(log_event) = serde_json::from_str::<LogEvent>(&line) {
            let message = log_event.message;
            let data = log_event
                .data
                .map(|d| serde_json::to_string_pretty(&d).unwrap())
                .map(|s| " ".to_string() + &s)
                .unwrap_or_default();
            match log_event.level {
                LogLevel::Debug => {
                    debug!(target: "driver", run = run_id.to_string(), "{}{}", message, data);
                }
                LogLevel::Info => {
                    info!(target: "driver", run = run_id.to_string(), "{}{}", message, data);
                }
                LogLevel::Warn => {
                    warn!(target: "driver", run = run_id.to_string(), "{}{}", message, data);
                }
                LogLevel::Error => {
                    error!(target: "driver", run = run_id.to_string(), "{}{}", message, data);
                }
            }
        } else {
            info!(target: "driver", run = run_id.to_string(), "{}", line);
        }
    }
    Ok(())
}

async fn read_stderr(
    mut stderr_reader: Lines<BufReader<ChildStderr>>,
    run_id: RunId,
) -> Result<()> {
    while let Some(line) = stderr_reader.next_line().await.unwrap() {
        error!(target = "driver", run = run_id.to_string(), "{}", line);
    }
    Ok(())
}
