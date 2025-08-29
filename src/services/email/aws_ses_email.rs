use crate::domain::Email;
use crate::domain::client::{EmailClient, EmailClientError};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_sesv2::{Client as SesClient, Error as SesError};
use secrecy::ExposeSecret;
use thiserror::Error;
use tracing::{error, info, instrument, warn};

#[derive(Debug, Error)]
pub enum SesEmailError {
    #[error("SES service error: {0}")]
    SesService(#[from] SesError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("AWS credentials are not found or invalid")]
    CredentialsError,
}

pub struct SesEmailClient {
    client: SesClient,
    from_email: String,
    region: String,
}

impl SesEmailClient {
    #[instrument(skip(from_email))]
    pub async fn new(
        region: &str,
        from_email: String,
    ) -> Result<Self, SesEmailError> {
        let region_obj = Region::new(region.to_string());

        info!("Initializing the SES client for a region: {}", region);

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_obj)
            .load()
            .await;

        let client = SesClient::new(&config);

        Self::validate_ses_access(&client).await?;

        info!("SES client initialized successfully with from_email: {}", from_email);

        Ok(Self {
            client,
            from_email,
            region: region.to_string(),
        })
    }

    async fn validate_ses_access(client: &SesClient) -> Result<(), SesEmailError> {
        match client.get_account().send().await {
            Ok(_) => {
                info!("Successfully validated access to SES service");
                Ok(())
            }
            Err(e) => {
                error!("Failed to access SES service: {}", e);
                Err(SesEmailError::SesService(e.into()))
            }
        }
    }

    async fn send_email_message(
        &self,
        email: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), SesEmailError> {
        info!("Sending email via SES to: {}", email.as_ref().expose_secret());

        let destination = aws_sdk_sesv2::types::Destination::builder()
            .to_addresses(email.as_ref().expose_secret())
            .build();

        let message_content = aws_sdk_sesv2::types::Content::builder()
            .data(content)
            .charset("UTF-8")
            .build()
            .map_err(|e| SesEmailError::Configuration(format!("Failed to build message content: {}", e)))?;

        let subject_content = aws_sdk_sesv2::types::Content::builder()
            .data(subject)
            .charset("UTF-8")
            .build()
            .map_err(|e| SesEmailError::Configuration(format!("Failed to build subject content: {}", e)))?;

        let body = aws_sdk_sesv2::types::Body::builder().text(message_content).build();

        let message = aws_sdk_sesv2::types::Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let response = self
            .client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(destination)
            .content(aws_sdk_sesv2::types::EmailContent::builder().simple(message).build())
            .send()
            .await
            .map_err(|e| SesEmailError::SesService(e.into()))?;

        info!(
            "Email sent successfully via SES. Message ID: {:?}",
            response.message_id()
        );

        Ok(())
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn email(&self) -> &str {
        &self.from_email
    }
}

#[async_trait::async_trait]
impl EmailClient for SesEmailClient {
    async fn send_email(
        &self,
        email: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError> {
        self.send_email_message(email, subject, content)
            .await
            .map_err(|e| EmailClientError::UnexpectedError(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretBox;

    #[tokio::test]
    #[ignore]
    async fn test_ses_client_creation_integration() {
        let result = SesEmailClient::new("us-east-1", "no-reply@example.com".to_string()).await;

        match result {
            Ok(client) => {
                println!("SES client created successfully");
                println!("Region: {}", client.region());
                println!("From Email: {}", client.email());
            }
            Err(e) => {
                eprintln!("Failed to create SES client: {}", e);
                if std::env::var("CI").is_ok() {
                    println!("Skipping test in CI environment");
                } else {
                    panic!("Failed to create SES client: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_email_content_building() {
        let subject = "Test Subject";
        let content = "Test content with special characters: \"quotes\" and 'apostrophes'";

        // Test that we can create the content structures
        let message_content = aws_sdk_sesv2::types::Content::builder()
            .data(content)
            .charset("UTF-8")
            .build();

        let subject_content = aws_sdk_sesv2::types::Content::builder()
            .data(subject)
            .charset("UTF-8")
            .build();

        assert!(message_content.is_ok());
        assert!(subject_content.is_ok());

        let message_content = message_content.unwrap();
        let subject_content = subject_content.unwrap();

        assert_eq!(message_content.data(), content);
        assert_eq!(subject_content.data(), subject);
        assert_eq!(message_content.charset().unwrap(), "UTF-8");
        assert_eq!(subject_content.charset().unwrap(), "UTF-8");
    }

    #[test]
    fn test_ses_email_error_display() {
        let config_error = SesEmailError::Configuration("Test config error".to_string());
        assert_eq!(config_error.to_string(), "Configuration error: Test config error");

        let creds_error = SesEmailError::CredentialsError;
        assert_eq!(creds_error.to_string(), "AWS credentials are not found or invalid");
    }
}
