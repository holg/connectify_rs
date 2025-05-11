

# Connectify Calendly Integration

**connectify-calendly** provides OAuth2 authentication with Calendly and calendar slot management for Connectify services, built on Actix Web and the OAuth2 crate.

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Endpoints](#api-endpoints)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features
- OAuth2 flow to obtain and refresh Calendly access tokens
- Secure CSRF state handling via signed cookies
- Persisted token storage via pluggable `TokenStore`
- Fetch available scheduling slots and book events
- Optional feature gate `calendly` to compile Calendly support

## Prerequisites
- Rust (1.65+)
- Cargo
- A Calendly OAuth client (Client ID, Client Secret, Redirect URI)
- A database URL for storing tokens (e.g., SQLite, Postgres)

## Installation
In your `Cargo.toml`:
```toml
connectify-calendly = { path = "../connectify_calendly", features = ["calendly"] }
```
Enable the `calendly` feature on your service crate to include Calendly handlers.

## Configuration
Calendly settings are loaded from environment variables in `CalendlyConfig::load()`:
```text
CALENDLY_CLIENT_ID        # OAuth2 client ID
CALENDLY_CLIENT_SECRET    # OAuth2 client secret
CALENDLY_REDIRECT_URI     # OAuth2 redirect URL
CSRF_STATE_SECRET         # 32+ byte secret key for signing CSRF cookies
DATABASE_URL              # Token store database URL
ENCRYPTION_KEY            # Hex-encoded key for encrypting tokens
CALENDLY_PERSONAL_TOKEN   # (optional) personal access token
```
Ensure `.env` or your environment provides these variables before starting the server.

## Usage
```rust
use actix_web::{App, HttpServer};
use connectify_calendly::{
    start_calendly_auth,
    calendly_auth_callback,
    get_available_slots,
    book_slot,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load config and create token store
    let cfg = connectify_calendly::config::CalendlyConfig::load().expect("config");
    let store = connectify_calendly::config::create_token_store(&cfg).await;
    let slots_state = connectify_calendly::config::create_slots_state(&cfg);

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(cfg.clone()))
            .app_data(actix_web::web::Data::new(store.clone()))
            .route("/auth/calendly/start", actix_web::web::get().to(start_calendly_auth))
            .route("/api/calendly/auth/", actix_web::web::get().to(calendly_auth_callback))
            .route("/slots", actix_web::web::get().to(get_available_slots))
            .route("/slots/book", actix_web::web::post().to(book_slot))
            // Add static test page if desired
            .service(connectify_calendly::calendly_test_file)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

## API Endpoints
| Method | Path                      | Description                                   |
| ------ | ------------------------- | --------------------------------------------- |
| GET    | `/auth/calendly/start`    | Redirects user to Calendly OAuth consent page |
| GET    | `/api/calendly/auth/`     | OAuth callback; exchanges code for token      |
| GET    | `/slots`                  | List available scheduling slots               |
| POST   | `/slots/book`             | Book a slot (JSON body)                       |
| GET    | `/calendly_test.html`     | Static test HTML for end-to-end testing       |

## Examples
```bash
# Start OAuth flow (in browser)
http://localhost:8080/auth/calendly/start

# After redirect, Calendly will call your `/api/calendly/auth/?code=...&state=...`

# Fetch slots
curl http://localhost:8080/slots?start_date=2025-05-01&end_date=2025-05-02

# Book a slot
curl -X POST http://localhost:8080/slots/book \
  -H 'Content-Type: application/json' \
  -d '{"start_time":"2025-05-02T10:00:00Z","end_time":"2025-05-02T10:30:00Z","summary":"Meeting"}'
```

## Contributing
1. Fork the repo
2. Create a feature branch
3. Run `cargo fmt` and `cargo clippy`
4. Add tests and update docs
5. Submit a pull request

## License
MIT OR Apache-2.0