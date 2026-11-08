//! An HTTP client for querying the Mojang API for server-side information.
//!
//! This file is taken from my previous project, Rustiny: https://github.com/dec4234/Rustiny/blob/master/src/api/ApiClient.rs

use log::{debug, error, info, trace};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use thiserror::Error;
use tokio::sync::Mutex;

/// An HTTP client used to interact with Mojang's API
pub(crate) struct MojangServerQueryClient;

impl MojangServerQueryClient {
	/// A simple GET request to an endpoint
	pub(crate) async fn get(url: String) -> Result<String, HttpError> {
		MojangServerQueryClient::get_params(url, HashMap::new()).await
	}

	/// A simple GET request with a map of query parameters
	pub(crate) async fn get_params(url: String, map: HashMap<&str, &str>) -> Result<String, HttpError> {
		let client = Client::new();
		let resp = client.get(&url).query(&map).send().await?;

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
	pub(crate) async fn get_parse<T: DeserializeOwned>(url: String, dewrap: bool) -> Result<T, HttpError> {
		if dewrap {
			let val = serde_json::from_str::<Value>(MojangServerQueryClient::get(url.clone()).await?.as_str())?;

			return Ok(serde_json::from_value::<T>(val)?);
		}

		let text = MojangServerQueryClient::get(url.clone()).await?;

		let r = serde_json::from_str::<T>(text.as_str())?;

		Ok(r)
	}

	/// GET to an endpoint with a map of query parameters and then convert to the specified type upon return
	///
	/// When dewrap is true, the outer JSON object will be removed prior to parsing
	pub(crate) async fn get_parse_params<T: DeserializeOwned>(url: String, dewrap: bool, map: HashMap<&str, &str>) -> Result<T, HttpError> {
		if dewrap {
			let val = serde_json::from_str::<Value>(MojangServerQueryClient::get_params(url.clone(), map).await?.as_str())?;

			return Ok(serde_json::from_value::<T>(val)?);
		}

		let text = MojangServerQueryClient::get_params(url, map).await?;

		Ok(serde_json::from_str::<T>(text.as_str())?)
	}

	/// A simple POST to an endpoint
	pub(crate) async fn post<S: Into<String>>(url: S, body: S) -> Result<String, HttpError> {
		MojangServerQueryClient::post_params(url, body, HashMap::new()).await
	}

	/// A simple POST to an endpoint with a map of query parameters
	pub(crate) async fn post_params<S: Into<String>>(url: S, body: S, map: HashMap<&str, &str>) -> Result<String, HttpError> {
		let url = url.into();
		let body = body.into();

		let client = Client::new();
		let resp = client.post(&url).body(body.clone()).header("Content-Type", "application/json").query(&map).send().await?;

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
	pub(crate) async fn post_parse<T: DeserializeOwned, S: Into<String>>(url: S, body: S, dewrap: bool) -> Result<T, HttpError> {
		if dewrap {
			let val = serde_json::from_str::<Value>(MojangServerQueryClient::post(url.into(), body.into()).await?.as_str())?;

			return Ok(serde_json::from_value::<T>(val)?);
		}

		let text = MojangServerQueryClient::post(url.into(), body.into()).await?;

		let r = serde_json::from_str::<T>(text.as_str())?;

		Ok(r)
	}

	/// POST to an endpoint with a map of query parameters and then convert to the specified type upon return
	///
	/// When dewrap is true, the outer JSON object will be removed prior to parsing
	pub(crate) async fn post_parse_params<T: DeserializeOwned, S: Into<String>>(url: S, body: S, map: HashMap<&str, &str>, dewrap: bool) -> Result<T, HttpError> {
		if dewrap {
			let val = serde_json::from_str::<Value>(MojangServerQueryClient::post_params(url.into(), body.into(), map).await?.as_str())?;

			return Ok(serde_json::from_value::<T>(val)?);
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
	#[error("Mojang returned an empty response (HTTP 204): the player is not authenticated / has no matching join record")]
	NotAuthenticated,
	#[error(transparent)]
	UuidError(#[from] uuid::Error),
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockito::Matcher;
	use serde::Deserialize;

	/// Query values such as a username or server ID must arrive URL-encoded. If they are not,
	/// a value with `&` or a space would split into wrong parameters and Mojang would check
	/// the wrong player.
	#[tokio::test]
	async fn get_params_url_encodes_query_values() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/q")
			.match_query(Matcher::AllOf(vec![Matcher::UrlEncoded("a".into(), "x y&z".into()), Matcher::UrlEncoded("b".into(), "1".into())]))
			.with_body("ok")
			.create_async()
			.await;

		let map = HashMap::from([("a", "x y&z"), ("b", "1")]);
		let text = MojangServerQueryClient::get_params(format!("{}/q", server.url()), map).await.unwrap();

		assert_eq!(text, "ok");
		mock.assert_async().await;
	}

	/// A non-2xx answer (rate limit, unknown player, outage) must be an error. If the error
	/// page body were returned as `Ok`, callers would try to use it as real data.
	#[tokio::test]
	async fn get_non_success_status_is_error() {
		let mut server = mockito::Server::new_async().await;
		server.mock("GET", "/q").with_status(429).with_body(r#"{"error":"TooManyRequests"}"#).create_async().await;

		let err = MojangServerQueryClient::get(format!("{}/q", server.url())).await.unwrap_err();

		match err {
			HttpError::StatusCode(msg) => assert!(msg.contains("429"), "message should name the status: {msg}"),
			other => panic!("expected StatusCode, got {other:?}"),
		}
	}

	/// HTTP 204 is a success code. `get` must return the empty body so that callers such as
	/// `has_joined` can tell "not authenticated" apart from a failed request.
	#[tokio::test]
	async fn get_no_content_is_ok_empty() {
		let mut server = mockito::Server::new_async().await;
		server.mock("GET", "/q").with_status(204).create_async().await;

		let text = MojangServerQueryClient::get(format!("{}/q", server.url())).await.unwrap();

		assert_eq!(text, "");
	}

	/// Mojang's JSON endpoints reject a POST that is not labelled as JSON. The body must also
	/// arrive unchanged, because it carries the access token and server ID.
	#[tokio::test]
	async fn post_sends_json_content_type_and_body() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("POST", "/p")
			.match_header("content-type", "application/json")
			.match_body(Matcher::Exact(r#"{"k":"v"}"#.into()))
			.with_status(204)
			.create_async()
			.await;

		MojangServerQueryClient::post(format!("{}/p", server.url()), r#"{"k":"v"}"#.to_string()).await.unwrap();

		mock.assert_async().await;
	}

	/// The non-2xx check also applies to POST, so a rejected join cannot look like success.
	#[tokio::test]
	async fn post_non_success_status_is_error() {
		let mut server = mockito::Server::new_async().await;
		server.mock("POST", "/p").with_status(403).create_async().await;

		let err = MojangServerQueryClient::post(format!("{}/p", server.url()), String::new()).await.unwrap_err();

		assert!(matches!(err, HttpError::StatusCode(_)));
	}

	#[derive(Deserialize, Debug)]
	struct Named {
		name: String,
	}

	/// A 200 response with a body that is not the expected JSON must be a `SerdeError`, not
	/// a panic or a default value.
	#[tokio::test]
	async fn get_parse_reports_malformed_json() {
		let mut server = mockito::Server::new_async().await;
		server.mock("GET", "/q").with_body("<html>maintenance</html>").create_async().await;

		let err = MojangServerQueryClient::get_parse::<Named>(format!("{}/q", server.url()), false).await.unwrap_err();

		assert!(matches!(err, HttpError::SerdeError(_)));
	}

	#[tokio::test]
	async fn get_parse_decodes_json() {
		let mut server = mockito::Server::new_async().await;
		server.mock("GET", "/q").with_body(r#"{"name":"dec4234"}"#).create_async().await;

		let named = MojangServerQueryClient::get_parse::<Named>(format!("{}/q", server.url()), false).await.unwrap();

		assert_eq!(named.name, "dec4234");
	}
}
