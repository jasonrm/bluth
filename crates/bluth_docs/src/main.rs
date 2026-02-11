mod assets;
mod layout;
mod pages;
mod ticker;

use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    lambda_http::tracing::init_default_subscriber();

    let app = Router::new()
        .route("/", get(pages::home))
        .route("/assets/{name}", get(assets::serve))
        .route("/ticker", get(ticker::sse_ticker));

    lambda_http::run_with_streaming_response(app).await
}
