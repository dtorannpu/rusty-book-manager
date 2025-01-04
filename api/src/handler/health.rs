use axum::{extract::State, http::StatusCode};
use registry::AppRegistry;

#[cfg_attr(debug_assertions, utoipa::path(get, path = "/api/v1/health"))]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[cfg_attr(debug_assertions, utoipa::path(get, path = "/api/v1/health/db"))]
pub async fn health_check_db(State(repository): State<AppRegistry>) -> StatusCode {
    if repository.health_check_repository().check_db().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
