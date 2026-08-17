mod assets;
mod error;
mod layout;
mod pages;
mod search;
mod ticker;

use crate::assets::Asset;
use crate::error::Error;
use crate::pages::Home;
use crate::search::Search;
use crate::ticker::Ticker;
use axum::Router;
use axum::routing::{get, post};

async fn run() -> Result<(), Error> {
    let app = Router::new()
        .route("/", get(Home::get))
        .route("/search", post(Search::post))
        .route("/assets/{name}", get(Asset::get))
        .route("/ticker", get(Ticker::stream));

    lambda_http::run_with_streaming_response(app)
        .await
        .map_err(Error::Lambda)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    match run().await {
        Ok(()) => Ok(()),
        Err(Error::Lambda(err)) => Err(err),
    }
}
