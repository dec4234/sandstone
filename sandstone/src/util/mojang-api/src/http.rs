//! An HTTP client for querying the Mojang API for server-side information.
//! 
//! This file is taken from my previous project, Rustiny: https://github.com/dec4234/Rustiny/blob/master/src/api/ApiClient.rs

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use log::{debug, error, info, trace};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;

/// An HTTP client used to interact with Mojang's API
pub struct MojangServerQueryClient;

impl MojangServerQueryClient {
    /// A simple GET request to an endpoint
    pub async fn get(url: String) -> Result<String, HttpError> {
        MojangServerQueryClient::get_params(url, HashMap::new()).await
    }

    /// A simple GET request with a map of query parameters
    pub async fn get_params(url: String, map: HashMap<&str, &str>) -> Result<String, HttpError> {
        let client = Client::new();
        let resp = client
            .get(&url)
            .query(&map)
            .send()
            .await?;

        if !resp.status().is_success() {
            error!("Failed to get from url: {}", url);
            return Err(HttpError::StatusCode(format!("Failed to get from url, code {}: {}", resp.status().as_str(), url)));
        }
        
        let text = resp.text().await?;

        trace!("GET {}\nResponse: \"{}\"", url, text.clone());

        Ok(text)
    }

    /// A simple GET request that also converts to the specified type upon a response
    /// 
    /// When dewrap is true, the outer JSON object will be removed prior to parsing
    pub async fn get_parse<T: DeserializeOwned>(url: String, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(MojangServerQueryClient::get(url.clone()).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = MojangServerQueryClient::get(url.clone()).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    /// GET to an endpoint with a map of query parameters and then convert to the specified type upon return
    /// 
    /// When dewrap is true, the outer JSON object will be removed prior to parsing
    pub async fn get_parse_params<T: DeserializeOwned>(url: String, dewrap: bool, map: HashMap<&str, &str>) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(MojangServerQueryClient::get_params(url.clone(), map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = MojangServerQueryClient::get_params(url, map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }

    /// A simple POST to an endpoint
    pub async fn post<S: Into<String>>(url: S, body: S) -> Result<String, HttpError> {
        MojangServerQueryClient::post_params(url, body, HashMap::new()).await
    }

    /// A simple POST to an endpoint with a map of query parameters 
    pub async fn post_params<S: Into<String>>(url: S, body: S, map: HashMap<&str, &str>) -> Result<String, HttpError> {
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
            error!("Failed to get from url: {}", url);
            return Err(HttpError::StatusCode(format!("Failed to post from url, code {}: {}", resp.status().as_str(), url)));
        }
        
        let text = resp.text().await?;

        trace!("POST {}\nBody - {}\nResponse: \"{}\"", url, body, text.clone());

        Ok(text)
    }

    /// A simple POST to an endpoint and a conversion to the specified type upon response
    ///
    /// When dewrap is true, the outer JSON object will be removed prior to parsing
    pub async fn post_parse<T: DeserializeOwned, S: Into<String>>(url: S, body: S, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(MojangServerQueryClient::post(url.into(), body.into()).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = MojangServerQueryClient::post(url.into(), body.into()).await?;

        let r = serde_json::from_str::<T>(text.as_str())?;

        Ok(r)
    }

    /// POST to an endpoint with a map of query parameters and then convert to the specified type upon return
    /// 
    /// When dewrap is true, the outer JSON object will be removed prior to parsing
    pub async fn post_parse_params<T: DeserializeOwned, S: Into<String>>(url: S, body: S, map: HashMap<&str, &str>, dewrap: bool) -> Result<T, HttpError> {
        if dewrap {
            let val = serde_json::from_str::<Value>(MojangServerQueryClient::post_params(url.into(), body.into(), map).await?.as_str())?;

            return Ok(serde_json::from_value::<T>(val)?)
        }

        let text = MojangServerQueryClient::post_params(url.into(), body.into(), map).await?;

        Ok(serde_json::from_str::<T>(text.as_str())?)
    }
}

/// Any sort of error that could occur while performing or processing the response of an HTTP request.
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