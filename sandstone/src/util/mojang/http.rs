use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;

/*
This file is taken from my previous project, Rustiny: https://github.com/dec4234/Rustiny/blob/master/src/api/ApiClient.rs
 */

pub struct ApiClient {
    debug_mode: Mutex<AtomicBool>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            debug_mode: Mutex::new(AtomicBool::new(false)),
        }
    }

    /// Enables Debug Mode
    ///
    /// Prints all requests and their responses as they come through
    /// usually only needed for development of the API but may be useful
    /// to someone wanting to learn the inner-workings of the system.
    pub async fn enable_debug_mode(self) -> Self {
        let mut temp = self.debug_mode.lock().await;
        *temp = AtomicBool::new(true);
        drop(temp);
        self
    }

    pub async fn is_debug_enabled(&self) -> bool {
        *self.debug_mode.lock().await.get_mut()
    }

    /// Clones the ApiClient
    pub async fn clone(&self) -> Self {
        Self {
            debug_mode: Mutex::new(AtomicBool::new(self.is_debug_enabled().await)),
        }
    }

    pub async fn get(&self, url: String) -> Result<String, HttpError> {
        self.get_params(url, HashMap::new()).await
    }

    pub async fn get_params(&self, url: String, map: HashMap<&str, &str>) -> Result<String, HttpError> {
        let client = Client::new();
        let resp = client
            .get(&url)
            .query(&map)
            .send()
            .await?;

        if !resp.status().is_success() {
            debug!("Failed to get from url: {}", url);
            return Err(HttpError::StatusCode(format!("Failed to get from url, code {}: {}", resp.status().as_str(), url)));
        }
        
        let text = resp.text().await?;

        if self.is_debug_enabled().await {
            println!("GET {}", url);
            println!("Response: \"{}\"", text.clone());
        }


        Ok(text)
    }

    pub async fn get_parse<T: DeserializeOwned>(&self, url: String, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.get(url.clone()).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.get(url.clone()).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    pub async fn get_parse_params<T: DeserializeOwned>(&self, url: String, dewrap: bool, map: HashMap<&str, &str>) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.get_params(url.clone(), map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.get_params(url, map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }

    pub async fn post<S: Into<String>>(&self, url: S, body: S) -> Result<String, HttpError> {
        self.post_params(url, body, HashMap::new()).await
    }

    pub async fn post_params<S: Into<String>>(&self, url: S, body: S, map: HashMap<&str, &str>) -> Result<String, HttpError> {
        let url = url.into();
        let body = body.into();
        
        let client = Client::new();
        let resp = client
            .post(&url)
            .body(body.clone())
            .header("Content-Type", "application/json")
            .query(&map)
            .send()
            .await?;

        if !resp.status().is_success() {
            debug!("Failed to get from url: {}", url);
            return Err(HttpError::StatusCode(format!("Failed to post from url, code {}: {}", resp.status().as_str(), url)));
        }
        
        let text = resp.text().await?;

        if self.is_debug_enabled().await {
            println!("POST {}", url);
            println!("Body - {}", body);
            println!("Response: \"{}\"", text.clone());
        }

        Ok(text)
    }

    pub async fn post_parse<T: DeserializeOwned, S: Into<String>>(&self, url: S, body: S, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.post(url.into(), body.into()).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.post(url.into(), body.into()).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    pub async fn post_parse_params<T: DeserializeOwned, S: Into<String>>(&self, url: S, body: S, map: HashMap<&str, &str>, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(self.post_params(url.into(), body.into(), map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = self.post_params(url.into(), body.into(), map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Received error code: {0}")]
    StatusCode(String),
}