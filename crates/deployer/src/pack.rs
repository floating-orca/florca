use anyhow::Result;
use florca_core::{function::FunctionEntity, lookup::LookupEntry};
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};
use tokio::fs::{self, File};

pub async fn pack_deployment(functions: Vec<FunctionEntity>) -> Result<File> {
    let work_dir = tempfile::tempdir()?;
    generate_lookup_file(&work_dir.path().join("lookup.json"), &functions).await?;
    query_blobs_and_write_files(&work_dir, functions).await?;
    let named_zip_file = NamedTempFile::with_suffix(".zip")?;
    zip_extensions::zip_create_from_directory(
        &named_zip_file.path().to_path_buf(),
        &work_dir.path().to_path_buf(),
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    let file = File::open(&named_zip_file).await?;
    Ok(file)
}

async fn generate_lookup_file(lookup_path: &Path, functions: &[FunctionEntity]) -> Result<()> {
    let entries: Vec<LookupEntry> = functions
        .iter()
        .map(|f| LookupEntry::from(f.raw()))
        .collect();
    fs::write(lookup_path, serde_json::to_string_pretty(&entries)?).await?;
    Ok(())
}

async fn query_blobs_and_write_files(
    work_dir: &TempDir,
    functions: Vec<FunctionEntity>,
) -> Result<()> {
    for function in functions {
        if let FunctionEntity::Plugin(plugin) = function {
            let file_path = work_dir.path().join(&plugin.location);
            fs::write(&file_path, &plugin.blob.unwrap()).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use florca_core::function::{FunctionEntity, RawFunctionEntity};
    use serde_json::json;
    use zip::ZipArchive;

    #[tokio::test]
    async fn test_pack_deployment() {
        let functions = vec![
            FunctionEntity::Plugin(RawFunctionEntity {
                id: 1,
                deployment_id: 1,
                name: "test_function".into(),
                kind: "plugin".to_string(),
                location: "test_location.ts".to_string(),
                hash: None,
                blob: Some(vec![1, 2, 3, 4]),
            }),
            FunctionEntity::Aws(RawFunctionEntity {
                id: 2,
                deployment_id: 1,
                name: "test_aws_function".into(),
                kind: "aws".to_string(),
                location: "arn::aws:lambda:eu-central-1:123456789012:function:test_deployment-test_aws_function".to_string(),
                hash: Some("12345".to_string()),
                blob: None,
            }),
        ];

        // Pack the deployment

        let file = pack_deployment(functions).await.unwrap();
        assert!(file.metadata().await.is_ok());

        // Extract the zip file for further verification

        let std_file = file.into_std().await;
        let temp_dir = TempDir::new().unwrap();
        let mut zip = ZipArchive::new(std_file).unwrap();
        zip.extract(temp_dir.path()).unwrap();

        // Verify the lookup file

        let lookup_path = temp_dir.path().join("lookup.json");
        assert!(lookup_path.exists());
        let content = fs::read_to_string(lookup_path).await.unwrap();
        let expected = json!([
            {
                "name": "test_function",
                "kind": "plugin",
                "location": "test_location.ts",
            },
            {
                "name": "test_aws_function",
                "kind": "aws",
                "location": "arn::aws:lambda:eu-central-1:123456789012:function:test_deployment-test_aws_function",
            }
        ]);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&content).unwrap(),
            expected
        );

        // Verify the plugin file

        let file_path = temp_dir.path().join("test_location.ts");
        assert!(file_path.exists());
        let file_content = fs::read(file_path).await.unwrap();
        assert_eq!(file_content, vec![1, 2, 3, 4]);
    }
}
