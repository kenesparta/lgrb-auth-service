use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct CaptchaVerificationRequest {
    secret: String,
    response: String,
    remoteip: Option<String>,
}

#[derive(Deserialize)]
struct CaptchaVerificationResponse {
    success: bool,
    challenge_ts: Option<String>,
    hostname: Option<String>,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
    score: Option<f64>,
    action: Option<String>,
}

pub struct CaptchaService {
    client: Client,
    secret_key: String,
}

impl CaptchaService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret_key = env::var("CAPTCHA_SECRET_KEY")
            .map_err(|_| "CAPTCHA_SECRET_KEY environment variable is required")?;

        Ok(Self {
            client: Client::new(),
            secret_key,
        })
    }

    pub async fn verify_captcha(
        &self,
        captcha_response: &str,
        client_ip: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let request = CaptchaVerificationRequest {
            secret: self.secret_key.clone(),
            response: captcha_response.to_string(),
            remoteip: client_ip.map(|ip| ip.to_string()),
        };

        let response = self
            .client
            .post("https://www.google.com/recaptcha/api/siteverify")
            .form(&request)
            .send()
            .await?;

        let verification_result: CaptchaVerificationResponse = response.json().await?;

        Ok(verification_result.success && verification_result.score.unwrap_or(0.0) >= 0.5)
    }
}
