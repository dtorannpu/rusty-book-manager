use adapter::database::connect_database_with;
use adapter::redis::RedisClient;
use anyhow::{Context, Result};
use api::route::{auth, v1};
use axum::http::Method;
use axum::Router;
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::{
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};
use registry::{AppRegistry, AppRegistryImpl};
use shared::config::AppConfig;
use shared::env::{which, Environment};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::{cors, LatencyUnit};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
use api::openapi::ApiDoc;
#[cfg(debug_assertions)]
use utoipa::OpenApi;
#[cfg(debug_assertions)]
use utoipa_redoc::{Redoc, Servable};

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

fn resource() -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, "book-manager"),
            KeyValue::new(SERVICE_VERSION, "1.0.0"),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
        ],
        SCHEMA_URL,
    )
}
fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    let host = std::env::var("JAEGER_HOST")?;
    let port = std::env::var("JAEGER_PORT")?;
    let endpoint = format!("http://{host}:{port}");
    println!("Jaeger endpoint {}", endpoint);

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    let tracer_provider = TracerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .build();
    let tracer = tracer_provider.tracer("tracing-otel-subscriber");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .json();

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}

async fn bootstrap() -> Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);
    let registry= AppRegistry(Arc::new(AppRegistryImpl::new(pool, kv, app_config)));
    let router = Router::new().merge(v1::routes()).merge(auth::routes());
    #[cfg(debug_assertions)]
    let router = router.merge(Redoc::with_url("/docs", ApiDoc::openapi()));
    let app = router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors())
        .with_state(registry);
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8080);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Unexpected error happened in server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            )
        })
}

async fn shutdown_signal() {
    fn purge_spans() {
        global::shutdown_tracer_provider();
    }
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await
            .expect("Failed to receive SIGTERM signal");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c=>{
            tracing::info!("Ctrl+C を受信しました。");
            purge_spans();
        },
        _ = terminate=>{
            tracing::info!("SIGTERM を受信しました。");
            purge_spans();
        },
    }
}
