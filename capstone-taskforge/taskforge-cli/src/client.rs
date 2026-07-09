use taskforge_core::Job;

/// A thin HTTP client for `taskforge-api` — the "thin client" half of the
/// "thin client, thick core" idea. This deliberately duplicates a small
/// amount of code that also exists in `taskforge-admin-bot/src/client.rs`
/// rather than introducing a shared `taskforge-client` crate for ~40 lines
/// of `reqwest` calls — see `README.md` for the reasoning, and as a
/// worthwhile stretch extension once/if a third client shows up.
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

    pub async fn enqueue_job(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        max_attempts: Option<u32>,
    ) -> Result<Job, String> {
        let body = serde_json::json!({
            "job_type": job_type,
            "payload": payload,
            "max_attempts": max_attempts,
        });
        let response = self
            .http
            .post(format!("{}/jobs", self.base_url))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("enqueue failed with status {}", response.status()));
        }
        response.json::<Job>().await.map_err(|e| e.to_string())
    }

    pub async fn get_job(&self, id: &str) -> Result<Job, String> {
        self.http
            .get(format!("{}/jobs/{}", self.base_url, id))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Job>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_jobs(
        &self,
        job_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Job>, String> {
        let mut url = format!("{}/jobs?limit={}", self.base_url, limit);
        if let Some(job_type) = job_type {
            url.push_str(&format!("&job_type={job_type}"));
        }
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
        let response = self
            .http
            .post(format!("{}/jobs/{}/cancel", self.base_url, id))
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
