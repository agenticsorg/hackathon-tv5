use anyhow::{Context, Result};
use reqwest::{Response, StatusCode};
use serde::Serialize;
use std::time::Duration;

pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl TestClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    pub fn set_auth(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
    }

    pub fn clear_auth(&mut self) {
        self.auth_token = None;
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        self.build_request(reqwest::Method::GET, path)
            .send()
            .await
            .context("GET request failed")
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.build_request(reqwest::Method::POST, path)
            .json(body)
            .send()
            .await
            .context("POST request failed")
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.build_request(reqwest::Method::PUT, path)
            .json(body)
            .send()
            .await
            .context("PUT request failed")
    }

    pub async fn patch<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.build_request(reqwest::Method::PATCH, path)
            .json(body)
            .send()
            .await
            .context("PATCH request failed")
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        self.build_request(reqwest::Method::DELETE, path)
            .send()
            .await
            .context("DELETE request failed")
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.get(path).await?;
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Request failed with status {}: {}", status, error_text);
        }

        response
            .json::<T>()
            .await
            .context("Failed to parse JSON response")
    }

    pub async fn post_json<T: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.post(path, body).await?;
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Request failed with status {}: {}", status, error_text);
        }

        response
            .json::<R>()
            .await
            .context("Failed to parse JSON response")
    }

    pub async fn expect_status(
        &self,
        response: Response,
        expected: StatusCode,
    ) -> Result<Response> {
        let status = response.status();
        if status != expected {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Expected status {}, got {}: {}",
                expected,
                status,
                error_text
            );
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TestClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.auth_token.is_none());
    }

    #[test]
    fn test_client_with_auth() {
        let client = TestClient::new("http://localhost:8080").with_auth("test-token");
        assert_eq!(client.auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_client_set_auth() {
        let mut client = TestClient::new("http://localhost:8080");
        client.set_auth("new-token");
        assert_eq!(client.auth_token, Some("new-token".to_string()));
    }

    #[test]
    fn test_client_clear_auth() {
        let mut client = TestClient::new("http://localhost:8080").with_auth("token");
        client.clear_auth();
        assert!(client.auth_token.is_none());
    }
}
