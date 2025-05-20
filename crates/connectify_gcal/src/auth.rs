// File: crates/connectify_gcal/src/auth.rs
use connectify_config::GcalConfig;
use google_calendar3::{
    hyper_rustls::{self, HttpsConnectorBuilder},
    hyper_util::client::legacy::connect::HttpConnector,
    hyper_util::client::legacy::Client,
    yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator},
    CalendarHub,
};
use std::{error::Error, path::Path};

// Type aliases for clarity
type Connector = hyper_rustls::HttpsConnector<HttpConnector>;

pub type HubType = CalendarHub<Connector>;

pub async fn create_calendar_hub(
    config: &GcalConfig,
) -> Result<HubType, Box<dyn Error + Send + Sync>> {
    let key_path = config
        .key_path
        .as_deref()
        .ok_or("Missing key_path in GcalConfig")?;

    let sa_key = read_service_account_key(Path::new(key_path)).await?;

    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;

    let https = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .build();

    // Create client without specifying body type
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);

    let hub = CalendarHub::new(client, auth);

    Ok(hub)
}
