use taskforge_core::Job;

/// A thin HTTP client for `taskforge-api` — this is the "thin client"
/// half of the "thin client, thick core" idea `taskforge-cli` also
/// demonstrates: all the actual domain logic (job state machine, storage,
/// worker behavior) lives elsewhere; this struct just makes HTTP calls and
/// deserializes JSON.
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        ApiClient {
            base_url: base_url.into(),
            token: token.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn list_jobs(&self, limit: usize) -> Result<Vec<Job>, String> {
        let url = format!("{}/jobs?limit={}", self.base_url, limit);
        self.http
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Job>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn cancel_job(&self, id: &str) -> Result<(), String> {
        let url = format!("{}/jobs/{}/cancel", self.base_url, id);
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("cancel failed with status {}", response.status()))
        }
    }
}
