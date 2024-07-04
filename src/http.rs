use reqwest::{Client, Method};
use revolt_models::v0::{DataMessageSend, Message};
use serde::{Deserialize, Serialize};
use futures::TryFutureExt;

#[derive(Deserialize, Debug, Clone)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub url: String,
    pub ws: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RevoltFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub voso: VoiceFeature,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RevoltConfig {
    pub revolt: String,
    pub features: RevoltFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
    pub build: BuildInformation,
}

#[derive(Clone)]
pub struct HttpClient {
    pub base: String,
    pub token: String,
    pub inner: Client
}

impl HttpClient {
    pub fn new(base: String, token: String) -> Self {
        HttpClient { base, token, inner: Client::new() }
    }

    async fn request<I: Serialize, O: for<'a> Deserialize<'a>>(&self, method: Method, route: impl AsRef<str>, body: Option<&I>) -> Result<O, reqwest::Error> {
        let mut builder = self.inner.request(method, format!("{}{}", &self.base, route.as_ref()))
            .header("x-session-token", &self.token);

        if let Some(body) = body {
            builder = builder.json(body)
        };

        builder.send()
            .and_then(|resp| {
                resp.json()
            }).await
    }

    pub async fn send_message(&self, channel: &str, body: &DataMessageSend) -> Result<Message, reqwest::Error> {
        self.request(Method::POST, format!("/{channel}/messages"), Some(body)).await
    }
}