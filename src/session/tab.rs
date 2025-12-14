use crate::{ prelude::*, TaskManager };

use reqwest::Client;
use serde_json::{ json, Value };

// The window tab
#[derive(Debug)]
pub struct Tab {
    pub(crate) client: Client,
    pub(crate) port: String,
    pub(crate) session_id: String,
    pub(crate) tab_id: String,
    pub(crate) url: String,
    pub(crate) manager: Arc<TaskManager>
}

impl Tab {
    /// Returns tab id
    pub fn get_id(&self) -> &str {
        &self.tab_id
    }
    
    /// Do tab active without locking other tasks
    async fn active_without_lock(&mut self) -> Result<()> {
        self.client
            .post(&fmt!("http://127.0.0.1:{}/session/{}/window", self.port, self.session_id))
            .json(&json!({"handle": self.tab_id }))
            .send()
            .await?;

        Ok(())
    }

    /// Do tab active
    pub async fn active(&mut self) -> Result<()> {
        // lock other tasks:
        self.manager.lock().await;
        
        // do tab active:
        self.active_without_lock().await?;

        // unlock other tasks:
        self.manager.unlock().await;

        Ok(())
    }
    
    /// Open URL-address
    pub async fn open<S>(&mut self, url: S) -> Result<()>
    where
        S: Into<String>
    {       
        let url = url.into();

        // lock other tasks:
        self.manager.lock().await;

        // do tab active:
        self.active_without_lock().await?;

        // loading URL:
        self.client
            .post(&fmt!("http://127.0.0.1:{}/session/{}/url", self.port, self.session_id))
            .json(&json!({ "url": url }))
            .send()
            .await?;

        // update url:
        self.url = url;

        // unlock other tasks:
        self.manager.unlock().await;

        Ok(())
    }

    /// Inject JavaScript to window tab
    pub async fn inject<D: serde::de::DeserializeOwned>(&mut self, script: &str) -> Result<D> {
        // lock other tasks:
        self.manager.lock().await;
        
        // do tab active:
        self.active_without_lock().await?;

        // execute script:
        let url = fmt!("http://127.0.0.1:{}/session/{}/execute/sync", self.port, self.session_id);
        let response = self.client
            .post(&url)
            .json(&json!({
                "script": script,
                "args": []
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        // unlock other tasks:
        self.manager.unlock().await;

        let value = response.get("value")
            .ok_or(Error::UnexpectedResponse)?
            .to_owned();
        
        Ok(serde_json::from_value::<D>(value)?)
    }

    /// Close window tab
    pub async fn close(&mut self) -> Result<()> {
        // lock other tasks:
        self.manager.lock().await;

        // do tab active:
        self.active_without_lock().await?;

        // close tab:
        loop {
            // do tab active (if tab not exists - break):
            if let Err(_) = self.active_without_lock().await {
                break;
            }

            // close tab:
            self.client
                .delete(&fmt!("http://127.0.0.1:{}/session/{}/window", self.port, self.session_id))
                .send()
                .await?;

            // check tab for closed:
            let handles_url = fmt!("http://127.0.0.1:{}/session/{}/window/handles", self.port, self.session_id);
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
                .map(|v| v.as_str().unwrap_or(""))
                .collect::<Vec<_>>();

            // tab not exists - success!
            if !handles.contains(&self.tab_id.as_str()) {
                break;
            }
        }

        // unlock other tasks:
        self.manager.unlock().await;

        Ok(())
    }
}
