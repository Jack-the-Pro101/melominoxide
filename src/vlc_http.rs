use reqwest::blocking::Client;
use serde::Deserialize;
use std::process::Command;
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
    reqwest_client: Client,
    host: String,
    port: String,
    password: String,
}

impl VlcHttpClient {
    pub fn new(host: &str, port: u16, password: &str) -> Self {
        let reqwest_client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("Failed to build reqwest client");

        VlcHttpClient {
            reqwest_client,
            host: host.to_string(),
            port: port.to_string(),
            password: password.to_string(),
        }
    }

    pub fn spawn_vlc_if_needed(
        &self,
        vlc_path: &str,
        playlist_path: &str,
    ) -> Result<(), Option<i32>> {
        if self.check_vlc_running() {
            println!("VLC already running, continuing...");
            return Ok(());
        }

        println!("VLC not running, spawning...");

        // Start VLC with HTTP interface enabled. Non-blocking spawn.
        let args = [
            "--extraintf",
            "http",
            "--http-host",
            self.host.as_str(),
            "--http-port",
            self.port.as_str(),
            "--http-password",
            self.password.as_str(),
            "--intf",
            "qt",
            "--random",
            "--loop",
            "--audio-filter",
            "compressor",
            "--compressor-rms-peak",
            "0.2",
            "--compressor-attack",
            "25.0",
            "--compressor-release",
            "101.0",
            "--compressor-threshold",
            "-22.4",
            "--compressor-ratio",
            "10.0",
            "--compressor-knee",
            "4.5",
            "--compressor-makeup-gain",
            "7.0",
            playlist_path,
        ];

        match Command::new(vlc_path).args(&args).spawn() {
            Ok(child) => {
                println!("Spawned VLC (pid={})", child.id());
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to spawn VLC: {}", e);
                Err(Some(e.raw_os_error().unwrap_or(-1)))
            }
        }
    }

    pub fn check_vlc_running(&self) -> bool {
        let response = self.query_status();

        match response {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn wait_until_ready(&self, timeout: Duration) -> bool {
        let start = Instant::now();

        while start.elapsed() < timeout {
            if self.check_vlc_running() {
                return true;
            }

            sleep(Duration::from_millis(250));
        }

        false
    }

    pub fn query_status(&self) -> Result<VlcState, Box<dyn std::error::Error>> {
        let url = format!("http://{}:{}/requests/status.json", self.host, self.port);
        let response = self
            .reqwest_client
            .get(&url)
            .basic_auth("", Some(self.password.clone()))
            .send()?;

        let state: VlcState = response.json()?;
        Ok(state)
    }
}
