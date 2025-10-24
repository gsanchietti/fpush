use serde::{Deserialize, Serialize};

/// Configuration for Acrobits singlepush API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcrobitsConfig {
    /// Acrobits singlepush endpoint URL (default: https://pnm.cloudsoftphone.com/pnm2/send)
    endpoint: String,
    /// Application ID for Acrobits
    app_id: String,
}

impl AcrobitsConfig {
    pub fn new(endpoint: String, app_id: String) -> Self {
        Self { endpoint, app_id }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }
}

impl Default for AcrobitsConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://pnm.cloudsoftphone.com/pnm2/send".to_string(),
            app_id: String::new(),
        }
    }
}
