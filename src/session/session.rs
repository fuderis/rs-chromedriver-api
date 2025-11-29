use crate::{ prelude::*, TaskManager };
use super::Tab;

use std::process::{ Command, Stdio };
use reqwest::Client;
use serde_json::{ json, Value };

/// The chromedriver session
#[derive(Debug)]
pub struct Session {
    client: Client,
    port: String,
    process: std::process::Child,
    session_id: String,
    manager: Arc<TaskManager>,
    is_first_tab: bool,
}

impl Session {
    /// Run chromedriver session in new window
    /// * port: a new chromedriver session IP-port
    /// * chromedriver_path: path to chromedriver (None = to use global PATH)
    /// * profile_path: path to storage user profile
    /// * headless: runs as headless mode (without interface)
    pub async fn run<S: Into<String>, P: Into<PathBuf>>(port: S, chromedriver_path: P, profile_path: Option<PathBuf>, headless: bool) -> Result<Self> {
        let port = port.into();
        
        // get path to chromedriver:
        let mut cmd = Command::new(chromedriver_path.into());

        // starting chromedriver server as background process:
        cmd.arg(fmt!("--port={port}"))
            .arg("--silent")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // settings for launching without a terminal window:
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.stdin(Stdio::null());
        }

        let process = cmd.spawn()?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;  // waiting when chromedriver is initializes..

        // init request options:
        let mut options = json!({
            "browserName": "chrome"
        });

        // loading & saving profile data + headless mode:
        let mut args = vec![];
        if let Some(path) = profile_path {
            let path = path
                .to_str()
                .ok_or(Error::InvalidPath)?
                .to_owned();

            args.push(format!("--user-data-dir={path}"));
        }

        // append headless mode:
        if headless {
            args.push("--headless".to_string());
            args.push("--disable-gpu".to_string());
        }

        options["goog:chromeOptions"] = json!({ "args": args });

        // init client:
        let client = Client::new();
        let session_url = fmt!("http://localhost:{port}/session");
        
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
            client,
            port,
            process,
            session_id,
            manager: Arc::new(TaskManager::new()),
            is_first_tab: true
        })
    }
    
    /// Open URL-address on new tab
    pub async fn open<S: Into<String>>(&mut self, url: S) -> Result<Arc<Mutex<Tab>>> {
        let url = url.into();

        // open default tab:
        let tab = if self.is_first_tab {
            self.is_first_tab = false;

            // get tabs list:
            let handles_url = format!("http://localhost:{}/session/{}/window/handles", self.port, self.session_id);
            let resp = self.client
                .get(&handles_url)
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?;

            let handles = resp["value"]
                .as_array()
                .ok_or(Error::IncorrectWindowHandles)?;

            // get first tab:
            let first_handle = handles
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or(Error::NoWindowHandles)?
                .to_string();

            // creating tab:
            let mut tab = Tab {
                client: self.client.clone(),
                port: self.port.clone(),
                session_id: self.session_id.clone(),
                window_handle: first_handle.clone(),
                url: url.clone(),
                manager: self.manager.clone()
            };

            // open URL:
            tab.open(url).await?;

            tab
        }
        // create a new tab:
        else {
            // opening new tab:
            let script = "window.open('about:blank', '_blank');";
            let execute_url = format!("http://localhost:{}/session/{}/execute/sync", self.port, self.session_id);
            self.client
                .post(&execute_url)
                .json(&json!({
                    "script": script,
                    "args": []
                }))
                .send()
                .await?
                .error_for_status()?;

            // get tabs list:
            let handles_url = format!("http://localhost:{}/session/{}/window/handles", self.port, self.session_id);
            let resp = self.client
                .get(&handles_url)
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?;

            let handles = resp["value"]
                .as_array()
                .ok_or(Error::IncorrectWindowHandles)?
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<_>>();

            // search new tab handle:
            let new_handle = handles.last().ok_or(Error::NoWindowHandles)?.clone();

            // create tab:
            let mut tab = Tab {
                client: self.client.clone(),
                port: self.port.clone(),
                session_id: self.session_id.clone(),
                window_handle: new_handle.clone(),
                url: url.clone(),
                manager: self.manager.clone()
            };
            
            // open URL:
            tab.open(url).await?;
            
            tab
        };

        Ok(Arc::new(Mutex::new(tab)))
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
