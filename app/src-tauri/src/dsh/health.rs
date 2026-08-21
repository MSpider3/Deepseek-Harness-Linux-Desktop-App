use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub node_healthy: bool,
    pub dsh_package_healthy: bool,
    pub webserver_reachable: bool,
    pub endpoint_url: Option<String>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

pub struct DshHealthChecker {
    client: Client,
}

impl DshHealthChecker {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Checks whether the DSH web server at the given port is reachable.
    pub async fn check_web_port(&self, port: u16) -> HealthStatus {
        let url = format!("http://127.0.0.1:{}", port);
        let start = std::time::Instant::now();

        match self.client.get(&url).send().await {
            Ok(resp) => {
                let latency = start.elapsed().as_millis() as u64;
                let status = resp.status();
                HealthStatus {
                    node_healthy: true,
                    dsh_package_healthy: true,
                    webserver_reachable: status.is_success() || status.as_u16() == 404,
                    endpoint_url: Some(url),
                    latency_ms: Some(latency),
                    error: None,
                }
            }
            Err(e) => HealthStatus {
                node_healthy: true,
                dsh_package_healthy: true,
                webserver_reachable: false,
                endpoint_url: Some(url),
                latency_ms: None,
                error: Some(e.to_string()),
            },
        }
    }
}
