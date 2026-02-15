use crate::error::SynapseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: i32,
    pub refresh_token: Option<String>,
}

pub async fn exchange_token(
    client_id: String,
    code: String,
    redirect_uri: String,
    code_verifier: String,
) -> Result<SpotifyTokenResponse, SynapseError> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("grant_type", "authorization_code".to_string()),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| SynapseError::Other(format!("Failed to send request: {}", e)))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(SynapseError::Other(format!(
            "Spotify API error: {}",
            err_text
        )));
    }

    let token_res: SpotifyTokenResponse = res
        .json()
        .await
        .map_err(|e| SynapseError::Other(format!("Failed to parse response: {}", e)))?;

    Ok(token_res)
}

pub async fn refresh_token(
    client_id: String,
    refresh_token: String,
) -> Result<SpotifyTokenResponse, SynapseError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];

    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| SynapseError::Other(format!("Failed to send request: {}", e)))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(SynapseError::Other(format!(
            "Spotify API error: {}",
            err_text
        )));
    }

    let token_res: SpotifyTokenResponse = res
        .json()
        .await
        .map_err(|e| SynapseError::Other(format!("Failed to parse response: {}", e)))?;

    Ok(token_res)
}
