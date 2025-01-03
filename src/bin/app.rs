use adapter::database::connect_database_with;
use adapter::redis::RedisClient;
use anyhow::{Context, Error, Result};
use api::route::{auth, v1};
use axum::http::Method;
use axum::Router;
use registry::AppRegistryImpl;
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

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    // let host = std::env::var("JAEGER_HOST")?;
    // let port = std::env::var("JAEGER_PORT")?;
    // let endpoint = format!("{host}:{port}");
    // let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
    //     .with_tonic()
    //     //.with_endpoint(endpoint)
    //     .build()?;
    // // Then pass it into provider builder
    // let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
    //     .with_simple_exporter(otlp_exporter)
    //     .with_batch_exporter(
    //         opentelemetry_otlp::SpanExporter::builder()
    //             .with_tonic()
    //             .build()?,
    //         opentelemetry_sdk::runtime::Tokio,
    //     )
    //     .with_resource(Resource::new(vec![KeyValue::new(
    //         "service.name",
    //         "book-manager",
    //     )]))
    //     .build();
    // let tracer = tracer_provider.tracer("sample");
    // let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .json();

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        //.with(opentelemetry)
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
    let registry = Arc::new(AppRegistryImpl::new(pool, kv, app_config));
    let app = Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
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
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
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
        .map_err(Error::from)
}

async fn shutdown_signal() {
    fn purge_spans() {
        //global::shutdown_tracer_provider();
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
        _= ctrl_c=>{
            tracing::info!("Ctrl+C を受信しました。");
            purge_spans();
        },
        _ = terminate=>{
            tracing::info!("SIGTERM を受信しました。");
            purge_spans();
        },
    }
}
