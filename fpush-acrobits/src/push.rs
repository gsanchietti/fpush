use fpush_traits::push::{PushError, PushResult, PushTrait};
use async_trait::async_trait;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use crate::config::AcrobitsConfig;

pub struct FpushAcrobits {
    client: reqwest::Client,
    config: AcrobitsConfig,
}

/// Request payload for Acrobits singlepush API
#[derive(Debug, Serialize, Deserialize)]
struct AcrobitsRequest {
    #[serde(rename = "verb")]
    verb: String,
    #[serde(rename = "AppId")]
    app_id: String,
    #[serde(rename = "DeviceToken")]
    device_token: String,
    #[serde(rename = "Message")]
    message: String,
}

/// Response from Acrobits singlepush API
#[derive(Debug, Deserialize)]
struct AcrobitsResponse {
    code: u16,
    #[allow(dead_code)]
    response: String,
}

impl FpushAcrobits {
    pub fn init(config: &AcrobitsConfig) -> PushResult<Self> {
        if config.app_id().is_empty() {
            error!("Acrobits app_id is not configured");
            return Err(PushError::CertLoading);
        }

        let client = reqwest::Client::new();
        
        Ok(Self {
            client,
            config: config.clone(),
        })
    }
}

#[async_trait]
impl PushTrait for FpushAcrobits {
    async fn send(&self, token: String) -> PushResult<()> {
        // Build the push request
        let request = AcrobitsRequest {
            verb: "NotifyGenericTextMessage".to_string(),
            app_id: self.config.app_id().to_string(),
            device_token: token.clone(),
            message: "New Message".to_string(),
        };

        debug!(
            "Sending Acrobits push to token: {}, endpoint: {}",
            token,
            self.config.endpoint()
        );

        // Send the request
        match self.client.post(self.config.endpoint()).json(&request).send().await {
            Ok(response) => {
                match response.json::<AcrobitsResponse>().await {
                    Ok(acrobits_response) => {
                        debug!(
                            "Received Acrobits response code: {} for token: {}",
                            acrobits_response.code, token
                        );
                        map_acrobits_response_to_error(acrobits_response.code)
                    }
                    Err(e) => {
                        error!("Failed to parse Acrobits response: {}", e);
                        Err(PushError::PushEndpointTmp)
                    }
                }
            }
            Err(e) => {
                error!("Failed to send request to Acrobits: {}", e);
                if e.is_timeout() || e.is_connect() {
                    Err(PushError::PushEndpointTmp)
                } else {
                    Err(PushError::PushEndpointTmp)
                }
            }
        }
    }
}

/// Map Acrobits response codes to PushError
fn map_acrobits_response_to_error(code: u16) -> PushResult<()> {
    match code {
        200 => Ok(()),
        // 404 means device token is no longer registered
        404 => Err(PushError::TokenBlocked),
        // 400 is a bad request (invalid AppId, missing fields)
        400 => Err(PushError::PushEndpointPersistent),
        // 500 is internal server error
        500 => Err(PushError::PushEndpointTmp),
        // Any other code is treated as unknown error
        code => {
            error!("Received unhandled error code from Acrobits: {}", code);
            Err(PushError::Unknown(code))
        }
    }
}
