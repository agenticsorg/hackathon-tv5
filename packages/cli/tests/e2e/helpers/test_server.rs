/// Mock HTTP server for testing URL fetching and network operations
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

/// Mock HTTP server state
#[derive(Clone)]
pub struct MockServerState {
    responses: Arc<Mutex<HashMap<String, MockResponse>>>,
}

#[derive(Clone)]
pub struct MockResponse {
    pub status: StatusCode,
    pub body: String,
    pub content_type: String,
}

impl MockServerState {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_response(&self, path: String, response: MockResponse) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(path, response);
    }

    pub fn add_html(&self, path: String, html: String) {
        self.add_response(
            path,
            MockResponse {
                status: StatusCode::OK,
                body: html,
                content_type: "text/html".to_string(),
            },
        );
    }

    pub fn add_404(&self, path: String) {
        self.add_response(
            path,
            MockResponse {
                status: StatusCode::NOT_FOUND,
                body: "Not Found".to_string(),
                content_type: "text/plain".to_string(),
            },
        );
    }
}

async fn handle_request(
    State(state): State<MockServerState>,
    uri: axum::http::Uri,
) -> Response {
    let path = uri.path().to_string();
    let responses = state.responses.lock().unwrap();

    if let Some(response) = responses.get(&path) {
        Response::builder()
            .status(response.status)
            .header("Content-Type", &response.content_type)
            .body(Body::from(response.body.clone()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
    }
}

/// Start a mock HTTP server on a random available port
pub async fn start_mock_server() -> (String, MockServerState) {
    let state = MockServerState::new();

    let app = Router::new()
        .fallback(handle_request)
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (url, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_returns_configured_responses() {
        let (url, state) = start_mock_server().await;

        state.add_html(
            "/test".to_string(),
            "<html><body>Test</body></html>".to_string(),
        );

        let client = reqwest::Client::new();
        let response = client.get(format!("{}/test", url)).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.unwrap();
        assert!(body.contains("Test"));
    }

    #[tokio::test]
    async fn test_mock_server_returns_404_for_unknown_paths() {
        let (url, _state) = start_mock_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/unknown", url))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
