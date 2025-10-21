// HTTP server module for fpush
// Provides a REST API with POST /fetch_messages endpoint for demo/testing purposes
//
// ## Configuration
//
// The HTTP server is started automatically when fpush runs. By default, it binds to
// 127.0.0.1:8080. You can change the bind address using the HTTP_BIND environment variable:
//
// ```bash
// HTTP_BIND=0.0.0.0:8080 ./fpush settings.json
// ```
//
// ## Testing with curl
//
// Successful request (with demo messages):
// ```bash
// curl -X POST http://127.0.0.1:8080/fetch_messages \
//   -H "Content-Type: application/json" \
//   -d '{"username":"user","password":"pass","last_id":"","last_sent_id":"","device":"device1"}'
// ```
//
// Request with existing last_id (no new messages):
// ```bash
// curl -X POST http://127.0.0.1:8080/fetch_messages \
//   -H "Content-Type: application/json" \
//   -d '{"username":"user","password":"pass","last_id":"123","last_sent_id":"456","device":"device1"}'
// ```
//
// Invalid credentials (empty username - returns 401):
// ```bash
// curl -X POST http://127.0.0.1:8080/fetch_messages \
//   -H "Content-Type: application/json" \
//   -d '{"username":"","password":"pass","last_id":"","last_sent_id":"","device":"device1"}'
// ```
//
// Wrong Content-Type (returns 415):
// ```bash
// curl -X POST http://127.0.0.1:8080/fetch_messages \
//   -H "Content-Type: text/plain" \
//   -d '{"username":"user","password":"pass","last_id":"","last_sent_id":"","device":"device1"}'
// ```

use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize)]
struct FetchMessagesRequest {
    username: String,
    password: String,
    #[serde(default)]
    last_id: Option<String>,
    #[serde(default)]
    last_sent_id: Option<String>,
    device: String,
}

#[derive(Debug, Serialize)]
struct FetchMessagesResponse {
    date: String,
    received_smss: Vec<SmsMessage>,
    sent_smss: Vec<SmsMessage>,
}

#[derive(Debug, Serialize, Clone)]
struct SmsMessage {
    sms_id: String,
    sending_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipient: Option<String>,
    sms_text: String,
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition_notification: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    displayed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_id: Option<String>,
}

async fn fetch_messages(
    req: HttpRequest,
    body: Option<web::Json<FetchMessagesRequest>>,
) -> HttpResponse {
    // Check Content-Type first
    if let Some(content_type) = req.headers().get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            // Allow application/json with or without charset
            if !ct_str.starts_with("application/json") {
                log::warn!("Unsupported Content-Type: {}", ct_str);
                return HttpResponse::UnsupportedMediaType()
                    .content_type("application/json")
                    .json(serde_json::json!({
                        "error": "Content-Type must be application/json"
                    }));
            }
        }
    } else {
        log::warn!("Missing Content-Type header");
        return HttpResponse::UnsupportedMediaType()
            .content_type("application/json")
            .json(serde_json::json!({
                "error": "Content-Type must be application/json"
            }));
    }

    // Get the body or return error if JSON parsing failed
    let body = match body {
        Some(json) => json,
        None => {
            log::warn!("Failed to parse JSON body");
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .json(serde_json::json!({
                    "error": "Invalid JSON body"
                }));
        }
    };

    // Validate credentials (non-empty username and password)
    if body.username.is_empty() || body.password.is_empty() {
        log::warn!("Invalid credentials: empty username or password");
        return HttpResponse::Unauthorized()
            .content_type("application/json")
            .json(serde_json::json!({
                "error": "Invalid credentials: username and password must not be empty"
            }));
    }

    log::info!(
        "Fetching messages for user: {}, device: {}",
        body.username,
        body.device
    );

    let now: DateTime<Utc> = Utc::now();
    let now_str = now.to_rfc3339();

    let mut received_smss = Vec::new();
    let mut sent_smss = Vec::new();

    // Return a demo received message if last_id is empty or None
    if body.last_id.is_none() || body.last_id.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
        let demo_received = SmsMessage {
            sms_id: "received-1001".to_string(),
            sending_date: "2024-01-15T10:30:00Z".to_string(),
            sender: Some("+1234567890".to_string()),
            recipient: None,
            sms_text: "Hello, this is a demo received message!".to_string(),
            content_type: "text/plain".to_string(),
            disposition_notification: Some(false),
            displayed: Some(false),
            stream_id: Some("stream-recv-1".to_string()),
        };
        received_smss.push(demo_received);
    }

    // Return a demo sent message if last_sent_id is empty or None
    if body.last_sent_id.is_none()
        || body
            .last_sent_id
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(false)
    {
        let demo_sent = SmsMessage {
            sms_id: "sent-2001".to_string(),
            sending_date: "2024-01-15T11:45:00Z".to_string(),
            sender: None,
            recipient: Some("+9876543210".to_string()),
            sms_text: "This is a demo sent message!".to_string(),
            content_type: "text/plain".to_string(),
            disposition_notification: Some(true),
            displayed: None,
            stream_id: Some("stream-sent-1".to_string()),
        };
        sent_smss.push(demo_sent);
    }

    // Sort messages by sending_date ascending (already in order for demo data)
    received_smss.sort_by(|a, b| a.sending_date.cmp(&b.sending_date));
    sent_smss.sort_by(|a, b| a.sending_date.cmp(&b.sending_date));

    let response = FetchMessagesResponse {
        date: now_str,
        received_smss,
        sent_smss,
    };

    log::info!(
        "Returning {} received and {} sent messages",
        response.received_smss.len(),
        response.sent_smss.len()
    );

    HttpResponse::Ok()
        .content_type("application/json")
        .json(response)
}

/// Start HTTP server on the specified bind address
/// This server provides a demo /fetch_messages endpoint for testing
pub async fn start_http_server(bind_addr: String) -> std::io::Result<()> {
    log::info!("Starting HTTP server on http://{}", bind_addr);
    log::info!("POST /fetch_messages endpoint is ready");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/fetch_messages", web::post().to(fetch_messages))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
