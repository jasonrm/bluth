mod assets;
mod layout;
mod pages;

use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    lambda_http::tracing::init_default_subscriber();

    let app = Router::new()
        .route("/", get(pages::home))
        .route("/assets/{name}", get(assets::serve));

    lambda_http::run(app).await
}
