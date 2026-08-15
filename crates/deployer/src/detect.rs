use crate::errors::DeployError;
use anyhow::Result;
use chksum::SHA2_256;
use florca_core::function::{FunctionConfig, FunctionName};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionToDeploy {
    Remote(RemoteFunctionToDeploy),
    Plugin(PluginFunctionToDeploy),
}

impl FunctionToDeploy {
    #[must_use]
    pub fn name(&self) -> &FunctionName {
        match self {
            Self::Remote(remote_function_to_deploy) => &remote_function_to_deploy.name,
            Self::Plugin(plugin_function_to_deploy) => &plugin_function_to_deploy.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFunctionToDeploy {
    pub name: FunctionName,
    pub path: PathBuf,
    pub hash: String,
    pub config: FunctionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginFunctionToDeploy {
    pub name: FunctionName,
    pub path: PathBuf,
}

/// # Panics
///
/// Panics if the name of a file is not valid Unicode
pub async fn detect_functions(path: &Path) -> Result<Vec<FunctionToDeploy>, DeployError> {
    let mut functions = Vec::new();
    let mut read_dir = tokio::fs::read_dir(path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            let function_path = entry.path();
            let function_toml_path = function_path.join("function.toml");
            if tokio::fs::metadata(&function_toml_path).await.is_ok() {
                let toml_string = tokio::fs::read_to_string(&function_toml_path).await?;
                functions.push(FunctionToDeploy::Remote(RemoteFunctionToDeploy {
                    name: entry.file_name().to_str().unwrap().to_string().into(),
                    path: function_path.clone(),
                    hash: hash_content(&function_path)?,
                    config: toml::from_str(&toml_string)
                        .map_err(|e| DeployError::InvalidFunctionConfig(e, function_toml_path))?,
                }));
            }
        } else if let Some(extension) = entry.path().extension()
            && extension == "ts"
        {
            functions.push(FunctionToDeploy::Plugin(PluginFunctionToDeploy {
                name: entry
                    .file_name()
                    .to_str()
                    .unwrap()
                    .trim_end_matches(".ts")
                    .to_string()
                    .into(),
                path: entry.path(),
            }));
        }
    }
    Ok(functions)
}

// The relative path makes renames change the hash. The length keeps file
// boundaries unambiguous.
fn hash_content(path: &Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();

    let mut hash = SHA2_256::new();
    for file in &files {
        let relative = file.strip_prefix(path)?;
        hash.update(relative.as_os_str().as_encoded_bytes());
        hash.update([0]);
        let content = std::fs::read(file)?;
        hash.update((content.len() as u64).to_le_bytes());
        hash.update(&content);
    }
    Ok(hash.digest().to_hex_lowercase())
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;
    use florca_core::function::{AwsFunctionConfig, FunctionName};

    #[test]
    fn test_hash_detects_renames() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();
        std::fs::write(path.join("a.py"), "content").unwrap();

        let before = hash_content(path).unwrap();
        std::fs::rename(path.join("a.py"), path.join("b.py")).unwrap();
        let after = hash_content(path).unwrap();

        assert_ne!(before, after, "Renaming a file should change the hash");
        assert_eq!(after, hash_content(path).unwrap(), "Hash should be stable");
    }

    #[tokio::test]
    async fn test_detect_functions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();

        // Create a sample function directory with a function.toml file

        let function_dir = path.join("sample_function");
        tokio::fs::create_dir(&function_dir).await.unwrap();
        let function_toml = function_dir.join("function.toml");
        let config = toml::toml! {
            provider = "aws"
            runtime = "nodejs24.x"
            handler = "index.handler"
            memory = 128
            timeout = 30
        };
        tokio::fs::write(&function_toml, toml::to_string(&config).unwrap())
            .await
            .unwrap();

        // Create a sample plugin file

        let plugin_file = path.join("sample_plugin.ts");
        tokio::fs::write(&plugin_file, "console.log('Hello, world!');")
            .await
            .unwrap();

        // Detect functions

        let functions = detect_functions(path).await.unwrap();
        assert_eq!(functions.len(), 2, "Two functions should be detected");

        // Verify that the plugin function is detected correctly

        let plugins_to_deploy = functions
            .iter()
            .filter_map(|f| {
                if let FunctionToDeploy::Plugin(plugin) = f {
                    Some(plugin)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            plugins_to_deploy.len(),
            1,
            "One plugin function should be detected"
        );
        let plugin_to_deploy = plugins_to_deploy.first().unwrap();
        assert_eq!(
            plugin_to_deploy.name,
            FunctionName::from("sample_plugin"),
            "Plugin function name should match"
        );
        assert!(
            plugin_to_deploy.path.ends_with("sample_plugin.ts"),
            "Plugin function path should match"
        );

        // Verify that the remote function is detected correctly

        let remote_functions_to_deploy = functions
            .iter()
            .filter_map(|f| {
                if let FunctionToDeploy::Remote(path) = f {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            remote_functions_to_deploy.len(),
            1,
            "One remote function should be detected"
        );
        let remote_function_to_deploy = remote_functions_to_deploy.first().unwrap();
        assert_eq!(
            remote_function_to_deploy.name,
            FunctionName::from("sample_function"),
            "Remote function name should match"
        );
        let FunctionConfig::Aws(AwsFunctionConfig {
            runtime,
            handler,
            memory,
            timeout,
        }) = &remote_function_to_deploy.config
        else {
            panic!("Expected AwsFunctionConfig");
        };
        assert_eq!(runtime, "nodejs24.x", "Runtime should match");
        assert_eq!(handler, "index.handler", "Handler should match");
        assert_eq!(*memory, 128, "Memory should match");
        assert_eq!(*timeout, 30, "Timeout should match");
    }
}
