use crate::prelude::*;
use super::*;

use std::process::{ Command, Stdio };
use reqwest::Client;
use serde_json::{ json, Value };

/// The chromedriver session
#[derive(Clone)]
pub struct Session {
    client: Client,
    port: u16,
    session_id: String,
    manager: Arc<SessionManager>
}

impl Session {
    /// Uses an already runned chromedriver
    pub async fn new<S: Into<String>>(port: u16, session_id: S) -> Result<Self> {
        // init client:
        let client = Client::new();

        Ok(Self {
            port,
            client,
            session_id: session_id.into(),
            manager: Arc::new(SessionManager::new())
        })
    }

    /// Returns chromederiver server port
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    /// Returns chromedriver sessions id
    pub fn get_id(&self) -> &str {
        &self.session_id
    }
    
    /// Run chromedriver session in new window
    /// * port: a new chromedriver session IP-port
    /// * chromedriver_path: path to chromedriver
    /// * profile_path: path to storage user profile (None = do not save session)
    /// * headless: runs as headless mode (without interface)
    pub async fn run<P: Into<PathBuf>>(port: u16, chromedriver_path: P, profile_path: Option<PathBuf>, headless: bool) -> Result<Self> {
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

        let _ = cmd.spawn()?;
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

            args.push(fmt!("--user-data-dir={path}"));
            args.push(fmt!("--disable-cache"));
            args.push(fmt!("--disk-cache-size=1"));
        }

        // append headless mode:
        if headless {
            args.push("--headless".to_string());
            args.push("--disable-gpu".to_string());
        }
        options["goog:chromeOptions"] = json!({ "args": args });

        // disable automation warning:
        #[cfg(feature = "no-automation")]
        {
            options["goog:chromeOptions"]["excludeSwitches"] = json!(["enable-automation"]);
            options["goog:chromeOptions"]["useAutomationExtension"] = json!(false);
            options["goog:chromeOptions"]["args"].as_array_mut().unwrap().extend([
                json!("--disable-blink-features=AutomationControlled"),
            ]);
        }

        // init client:
        let client = Client::new();
        let session_url = fmt!("http://127.0.0.1:{port}/session");
        
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
            .json::<Value>()
            .await?;

        // get session id:
        let session_id = response["value"]["sessionId"]
            .as_str()
            .ok_or(Error::IncorrectSessionId)?
            .to_string();

        #[allow(unused_mut)]
        let mut this = Self {
            client,
            port,
            session_id,
            manager: Arc::new(SessionManager::new()),
        };

        #[cfg(feature = "no-automation")]
        {
            this.disable_automation().await?;
        }
            
        Ok(this)
    }

    /// Disabled automation context
    #[cfg(feature = "no-automation")]
    async fn disable_automation(&mut self) -> Result<()> {
        let cdp_url = fmt!("http://127.0.0.1:{}/session/{}/chromium/send_command", self.port, self.session_id);
    
        let script = r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            delete navigator.__proto__.webdriver;
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1,2,3,4,5]
            });
        "#;
        
        let params = json!({
            "cmd": "Page.addScriptToEvaluateOnNewDocument",
            "params": { "source": script }
        });

        let resp = self.client
            .post(&cdp_url)
            .json(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if resp.get("error").is_some() {
            return Err(fmt!("CDP command failed: {resp:?}").into());
        }

        let exec_url = fmt!("http://127.0.0.1:{}/session/{}/execute/sync", self.port, self.session_id);
        let _ = self.client
            .post(&exec_url)
            .json(&json!({
                "script": script,
                "args": []
            }))
            .send()
            .await?
            .error_for_status();

        Ok(())
    }

    /// Returns all tab identifiers
    pub async fn get_tabs_ids(&self) -> Result<Vec<String>> {
        let handles_url = fmt!("http://127.0.0.1:{}/session/{}/window/handles", self.port, self.session_id);

        let response = self.client
            .get(&handles_url)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let handles = response["value"]
            .as_array()
            .ok_or(Error::IncorrectWindowHandles)?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or(Error::IncorrectWindowHandles)
                    .map(|s| s.to_string())
                    .map_err(|e| e.into())
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(handles)
    }

    /// Returns current active tab
    pub async fn get_active_tab(&self) -> Result<Arc<Mutex<Tab>>> {
        let url = fmt!("http://127.0.0.1:{}/session/{}/window", self.port, self.session_id);
    
        let resp = self.client
            .get(&url)
            .send()
            .await?
            .json::<Value>()
            .await?;
        
        let handle = resp["value"]
            .as_str()
            .ok_or(Error::IncorrectWindowHandle)?
            .to_string();
        
        match self.get_tab(&handle).await? {
            Some(tab) => Ok(tab),
            None => Err(Error::TabNotFound(handle).into())
        }
    }

    /// Returns all tabs
    pub async fn get_tabs(&self) -> Result<Vec<Arc<Mutex<Tab>>>> {
        let handles = self.get_tabs_ids().await?;
        
        let mut tabs = Vec::with_capacity(handles.len());
        for tab_id in handles {
            let tab = Tab {
                client: self.client.clone(),
                port: self.port,
                session_id: self.session_id.clone(),
                tab_id,
                url: String::new(),
                manager: self.manager.clone(),
            };
            tabs.push(Arc::new(Mutex::new(tab)));
        }
        
        Ok(tabs)
    }
    
    /// Returns tab by id
    pub async fn get_tab<S: Into<String>>(&self, tab_id: S) -> Result<Option<Arc<Mutex<Tab>>>> {
        let tab_id = tab_id.into();
        let handles = self.get_tabs_ids().await?;
        
        if !handles.contains(&tab_id) {
            return Ok(None);
        }
        
        let tab = Arc::new(Mutex::new(Tab {
            client: self.client.clone(),
            port: self.port,
            session_id: self.session_id.clone(),
            tab_id,
            url: String::new(),
            manager: self.manager.clone(),
        }));
        
        Ok(Some(tab))
    }
    
    /// Open URL-address on a new tab
    pub async fn open<S: Into<String>>(&self, url: S) -> Result<Arc<Mutex<Tab>>> {
        let url = url.into();

        // lock tabs activity:
        self.manager.lock().await;

        // activate last tab:
        {
            // get tab handles:
            let handles = self.get_tabs_ids().await?;
            
            // activate last tab:
            let last_handle = handles.last().ok_or(Error::NoWindowHandles)?.clone();
            self.client
                .post(&fmt!("http://127.0.0.1:{}/session/{}/window", self.port, self.session_id))
                .json(&json!({"handle": &last_handle}))
                .send()
                .await?;
        }
        
        // open new tab:
        let tab = {
            let execute_url = fmt!("http://127.0.0.1:{}/session/{}/execute/sync", self.port, self.session_id);
            self.client
                .post(&execute_url)
                .json(&json!({
                    "script": "window.open('about:blank', '_blank');",
                    "args": []
                }))
                .send()
                .await?;

            sleep(Duration::from_millis(100)).await;

            // get tabs list:
            let handles = self.get_tabs_ids().await?;

            // search new tab handle:
            let new_handle = handles.last().ok_or(Error::NoWindowHandles)?.clone();

            // create tab:
            let mut tab = Tab {
                client: self.client.clone(),
                port: self.port,
                session_id: self.session_id.clone(),
                tab_id: new_handle,
                url: str!(""),
                manager: self.manager.clone()
            };

            // unlock tabs:
            self.manager.unlock().await;
            
            // open URL:
            tab.open(url).await?;
            tab
        };

        // unlock tabs (if it's not already unlocked):
        self.manager.unlock().await;

        Ok(Arc::new(Mutex::new(tab)))
    }

    /// Close chromedriver session
    pub async fn close(&self) -> Result<()> {
        // close session:
        let close_url = fmt!("http://127.0.0.1:{}/session/{}", self.port, self.session_id);
        let _ = self.client
            .delete(&close_url)
            .send()
            .await;

        // shutdown server:
        let shutdown_url = fmt!("http://127.0.0.1:{}/shutdown", self.port);
        let _ = self.client.post(&shutdown_url).send().await;
        
        // additional quit:
        let quit_url = fmt!("http://127.0.0.1:{}/quit", self.port);
        let _ = self.client.post(&quit_url).send().await;

        Ok(())
    }
}
