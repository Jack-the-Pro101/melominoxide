use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
pub struct VlcState {
    pub state: Option<String>, // "playing", "paused", "stopped"
    pub length: Option<i64>,   // seconds
    pub position: Option<f64>, // 0.0 to 1.0
    pub information: Option<Information>,
}

#[derive(Debug, Deserialize)]
pub struct Information {
    pub category: Option<Categories>,
}

#[derive(Debug, Deserialize)]
pub struct Categories {
    pub meta: Option<Meta>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub title: Option<String>,
    pub filename: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub struct VlcHttpClient {
    client: Client,
    host: String,
    port: u16,
    password: String,
}

impl VlcHttpClient {
    pub fn new(host: &str, port: u16, password: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest client");

        VlcHttpClient {
            client,
            host: host.to_string(),
            port,
            password: password.to_string(),
        }
    }

    pub fn wait_until_ready(&self, timeout: Duration) -> bool {
        let url = format!("http://{}:{}/requests/status.json", self.host, self.port);
        let start = Instant::now();

        while start.elapsed() < timeout {
            let resp = self
                .client
                .get(&url)
                .basic_auth("", Some(self.password.clone()))
                .send();

            match resp {
                Ok(r) => {
                    if r.status() == StatusCode::OK {
                        return true;
                    }
                }
                Err(_) => {}
            }

            sleep(Duration::from_millis(250));
        }

        false
    }

    pub fn query_status(&self) -> Result<VlcState, Box<dyn std::error::Error>> {
        let url = format!("http://{}:{}/requests/status.json", self.host, self.port);
        let resp = self
            .client
            .get(&url)
            .basic_auth("", Some(self.password.clone()))
            .send()?;

        let state: VlcState = resp.json()?;
        Ok(state)
    }
}
