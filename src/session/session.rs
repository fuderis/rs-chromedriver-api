use crate::prelude::*;

use std::process::{ Command, Stdio };
use reqwest::Client;
use serde_json::{ json, Value };

/// The chromedriver session
#[derive(Debug)]
pub struct Session {
    process: std::process::Child,
    client: Client,
    port: String,
    session_id: String,
}

impl Session {
    /// Run chromedriver session && Browser window
    /// * port: a new chromedriver session port
    /// * profile_path: path to storage user profile
    pub async fn run<S>(port: S, profile_path: Option<&str>) -> Result<Self>
    where
        S: Into<String>
    {
        let port = port.into();
        
        // starting chromedriver server as background process:
        let mut cmd = Command::new("chromedriver");
        cmd.arg(fmt!("--port={port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let process = cmd.spawn()?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;  // waiting when chromedriver is initializes..

        // init client:
        let client = Client::new();
        let session_url = fmt!("http://localhost:{port}/session");

        // init request options:
        let mut options = json!({
            "browserName": "chrome"
        });

        // loading & saving profile data:
        if let Some(path) = profile_path {
            options["goog:chromeOptions"] = json!({ "args": vec![ fmt!("--user-data-dir={path}") ] });
        }

        // send request:
        let response = client
            .post(&session_url)
            .json(&json!({
                "capabilities": {
                    "alwaysMatch": options
                }
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        // get session id:
        let session_id = response["value"]["sessionId"]
            .as_str()
            .ok_or(Error::IncorrectSessionId)?
            .to_string();

        Ok(Self {
            process,
            client,
            port,
            session_id
        })
    }
    
    /// Open URL-address on Chrome browser window
    pub async fn open(&mut self, url: &str) -> Result<()> {
        // opening URL:
        self.client
            .post(&fmt!("http://localhost:{}/session/{}/url", self.port, self.session_id))
            .json(&json!({ "url": url }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Inject JavaScript code to process
    pub async fn inject(&self, script: &str) -> Result<Value> {
        let url = format!("http://localhost:{}/session/{}/execute/sync", self.port, self.session_id);
        let response = self.client
            .post(&url)
            .json(&json!({
                "script": script,
                "args": []
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(response["value"].clone())
    }

    /// Click to element by CSS selector
    pub async fn click(&self, selector: &str) -> Result<Value> {
        self.inject(&fmt!(r#"
            __button = document.querySelector('{selector}');
            __button.focus();
            __button.click();
        "#)).await
    }

    /// Change element value by CSS selector
    pub async fn value(&self, selector: &str, value: &str) -> Result<Value> {
        self.inject(&fmt!(r#"
            __input = document.querySelector('{selector}');
            __input.value = "{value}";
        "#)).await
    }

    /// Close chromedriver session
    pub async fn close(mut self) -> Result<()> {
        // closing window:
        let url = fmt!("http://localhost:{}/session/{}", self.port, self.session_id);
        self.client
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;

        // killing chromedriver background process:
        self.process.kill()?;

        Ok(())
    }
}
