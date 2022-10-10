use std::net::SocketAddr;
use axum::{
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use tokio::{self, sync::mpsc};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod supervisor;
mod bucket;
mod http_app;

fn add_observability_to(r: Router) -> Router {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "bidon=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    return r.layer(
        TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );
}


#[tokio::main]
async fn main() {
    let router = add_observability_to(Router::new());

    let bidon = supervisor::start_bidon().await;
    let app = http_app::mount_bidon_routes(router, bidon);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
