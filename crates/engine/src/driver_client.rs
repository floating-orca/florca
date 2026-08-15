use anyhow::Result;
use reqwest::Response;

pub(crate) async fn require_success(response: Response) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    anyhow::bail!("Driver responded with {}: {body}", status.as_u16())
}
