mod assets;
mod layout;
mod pages;
mod ticker;

use crate::assets::Asset;
use crate::pages::Home;
use crate::ticker::Ticker;
use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let app = Router::new()
        .route("/", get(Home::get))
        .route("/assets/{name}", get(Asset::get))
        .route("/ticker", get(Ticker::stream));

    lambda_http::run_with_streaming_response(app).await
}
