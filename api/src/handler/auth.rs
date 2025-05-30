use crate::extractor::AuthorizedUser;
use crate::model::auth::{AccessTokenResponse, LoginRequest};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use kernel::model::auth::event::CreateToken;
use registry::AppRegistry;
use shared::error::AppResult;

#[cfg_attr(debug_assertions, utoipa::path(post, path = "/auth/login"))]
#[tracing::instrument(skip(registry, req))]
pub async fn login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AccessTokenResponse>> {
    let user_id = registry
        .auth_repository()
        .verify_user(&req.email, &req.password)
        .await?;
    let access_token = registry
        .auth_repository()
        .create_token(CreateToken::new(user_id))
        .await?;

    Ok(Json(AccessTokenResponse {
        user_id,
        access_token: access_token.0,
    }))
}

#[cfg_attr(debug_assertions, utoipa::path(post, path = "/auth/logout"))]
#[tracing::instrument(skip(user, registry), fields(user_id = %user.user.id.to_string()))]
pub async fn logout(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    registry
        .auth_repository()
        .delete_token(user.access_token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
