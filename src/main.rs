use axum::{middleware, routing::get, Router};
use raito_proving_service::{
    handlers::{
        get_block_by_identifier, get_block_proof, get_blocks, get_header_status,
        get_transaction_status, health_check, metrics_handler, ApiDoc,
    },
    middleware::{cors_layer, metrics_middleware, security_headers_middleware},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    info!("Starting Raito Proving Service");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server will listen on {}", addr);

    let app = create_app();

    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app() -> Router {
    let api_routes = Router::new()
        .route("/blocks", get(get_blocks))
        .route("/blocks/:identifier", get(get_block_by_identifier))
        .route("/blocks/:height/proof", get(get_block_proof))
        .route("/tx/:txid", get(get_transaction_status))
        .route("/header/:hash", get(get_header_status))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(metrics_middleware))
                .layer(middleware::from_fn(security_headers_middleware)),
        );

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/healthz", get(health_check))
        .route("/metrics", get(metrics_handler))
        .nest("/v1", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer()),
        )
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,raito_proving_service=debug,tower_http=debug"));

    let format_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(format_layer)
        .init();

    info!("Tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::Value;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;
        response.assert_status_ok();

        let json: Value = response.json();
        assert_eq!(json["status"], "up");
    }

    #[tokio::test]
    async fn test_blocks_endpoint() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/v1/blocks").await;
        response.assert_status_ok();

        let json: Value = response.json();
        assert!(json["blocks"].is_array());
        assert!(json["total"].is_number());
    }

    #[tokio::test]
    async fn test_block_by_height() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/v1/blocks/869123").await;
        if response.status_code() == 200 {
            let json: Value = response.json();
            assert_eq!(json["height"], 869123);
        }
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let app = create_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;
        assert!(response
            .headers()
            .contains_key("access-control-allow-origin"));
    }
}
