use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::Mutex;

/*
This file is taken from my previous project, Rustiny: https://github.com/dec4234/Rustiny/blob/master/src/api/ApiClient.rs
 */

pub struct ApiClient {
    DEBUG_MODE: Mutex<AtomicBool>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            DEBUG_MODE: Mutex::new(AtomicBool::new(false)),
        }
    }

    /// Enables Debug Mode
    ///
    /// Prints all requests and their responses as they come through
    /// usually only needed for development of the API but may be useful
    /// to someone wanting to learn the inner-workings of the system.
    pub async fn enable_debug_mode(self) -> Self {
        let mut temp = self.DEBUG_MODE.lock().await;
        *temp = AtomicBool::new(true);
        drop(temp);
        self
    }

    pub async fn is_debug_enabled(&self) -> bool {
        *self.DEBUG_MODE.lock().await.get_mut()
    }

    /// Clones the ApiClient
    pub async fn clone(&self) -> Self {
        Self {
            DEBUG_MODE: Mutex::new(AtomicBool::new(self.is_debug_enabled().await)),
        }
    }

    pub async fn get(&self, url: String) -> Result<String> {
        self.get_params(url, HashMap::new()).await
    }

    pub async fn get_params(&self, url: String, map: HashMap<&str, &str>) -> Result<String> {
        let client = Client::new();
        let resp = client
            .get(url.clone())
            .query(&map)
            .send()
            .await?;

        if !resp.status().is_success() {
            debug!("Failed to get from url: {}", url);
            return Err(anyhow::anyhow!("Failed to get from url, code {}: {}", resp.status().as_str(), url));
        }
        
        let text = resp.text().await?;

        if self.is_debug_enabled().await {
            println!("GET {}", url);
            println!("Response: \"{}\"", text.clone());
        }


        Ok(text)
    }

    pub async fn get_parse<T: DeserializeOwned>(&self, url: String, dewrap: bool) -> Result<T> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.get(url.clone()).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.get(url.clone()).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    pub async fn get_parse_params<T: DeserializeOwned>(&self, url: String, dewrap: bool, map: HashMap<&str, &str>) -> Result<T> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.get_params(url.clone(), map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.get_params(url, map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }

    pub async fn post(&self, url: String, body: String) -> Result<String> {
        self.post_params(url, body, HashMap::new()).await
    }

    pub async fn post_params(&self, url: String, body: String, map: HashMap<&str, &str>) -> Result<String> {
        let client = Client::new();
        let resp = client
            .post(url.clone())
            .body(body.clone())
            .query(&map)
            .send()
            .await?;

        if !resp.status().is_success() {
            debug!("Failed to get from url: {}", url);
            return Err(anyhow::anyhow!("Failed to post from url, code {}: {}", resp.status().as_str(), url));
        }
        
        let text = resp.text().await?;

        if self.is_debug_enabled().await {
            println!("POST {}", url);
            println!("Body - {}", body);
            println!("Response: \"{}\"", text.clone());
        }

        Ok(text)
    }

    pub async fn post_parse<T: DeserializeOwned>(&self, url: String, body: String, dewrap: bool) -> Result<T> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.post(url, body).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.post(url, body).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    pub async fn post_parse_params<T: DeserializeOwned>(&self, url: String, body: String, map: HashMap<&str, &str>, dewrap: bool) -> Result<T> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.post_params(url, body, map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.post_params(url, body, map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }
}