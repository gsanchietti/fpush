mod config;
mod error;
mod xmpp;
mod http_server;

use fpush_push::FpushPush;

use log::{debug, error, info};
use std::sync::Arc;

/// init env_logger
fn setup_logging() {
    env_logger::init();
}

#[tokio::main]
async fn main() {
    setup_logging();
    tokio_rustls::rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // get settings name
    let args: Vec<String> = std::env::args().collect();
    let settings_filename = match args.get(1) {
        Some(f) => f,
        None => "./settings.json",
    };
    info!("Loading config file {}", settings_filename);

    let settings = match crate::config::load_config(settings_filename) {
        Ok(s) => {
            debug!("Config loaded");
            s
        }
        Err(e) => {
            panic!("Error loading config file: {}", e);
        }
    };

    let push_impl: Arc<FpushPush> = Arc::new(FpushPush::new(settings.push_modules()).await);

    // Start HTTP server in a separate task
    let http_bind_addr = std::env::var("HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    tokio::spawn(async move {
        if let Err(e) = http_server::start_http_server(http_bind_addr).await {
            error!("HTTP server error: {}", e);
        }
    });

    // Main XMPP connection loop
    loop {
        info!(
            "Opening connection to {}",
            settings.component().server_hostname()
        );
        // open component connection
        match crate::xmpp::init_component_connection(&settings).await {
            Err(e) => {
                error!("Could not connect to XMPP Server {}", e);
                info!(
                    "Waiting {} seconds before reconnecting",
                    settings.timeout().xmppconnection_error().as_secs()
                );
                tokio::time::sleep(*settings.timeout().xmppconnection_error()).await;
            }
            Ok(component) => {
                // open new messageLoop
                crate::xmpp::message_loop_main_thread(component, push_impl.clone()).await;
            }
        }
    }
}
