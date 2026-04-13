use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub session_percent: f32,
    pub session_reset_minutes: u32,
    pub weekly_percent: f32,
    pub weekly_reset_minutes: u32,
    pub plan_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    #[serde(default)]
    pub daily_limit_percent_used: f32,
    #[serde(default)]
    pub weekly_limit_percent_used: f32,
    #[serde(default)]
    pub daily_usage_limit_reset_timestamp: i64,
    #[serde(default)]
    pub weekly_usage_limit_reset_timestamp: i64,
    #[serde(default)]
    pub plan: String,
}

pub async fn fetch_usage(
    cookie: &str,
    org_id: &str,
) -> Result<UsageData, Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!(
        "https://api.claude.ai/api/organizations/{}/usage",
        org_id
    );

    let response = client
        .get(&url)
        .header("Cookie", format!("sessionKey={}", cookie))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if response.status().is_client_error() || response.status().is_server_error() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let api_response: ApiResponse = response.json().await?;

    let now = chrono::Utc::now().timestamp();
    let session_reset_secs = api_response.daily_usage_limit_reset_timestamp - now;
    let weekly_reset_secs = api_response.weekly_usage_limit_reset_timestamp - now;

    Ok(UsageData {
        session_percent: api_response.daily_limit_percent_used,
        session_reset_minutes: (session_reset_secs.max(0) / 60) as u32,
        weekly_percent: api_response.weekly_limit_percent_used,
        weekly_reset_minutes: (weekly_reset_secs.max(0) / 60) as u32,
        plan_type: api_response.plan,
    })
}
