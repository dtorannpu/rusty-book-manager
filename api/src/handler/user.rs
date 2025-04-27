use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::UserId, user::event::DeleteUser};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::model::checkout::CheckoutsResponse;
use crate::{
    extractor::AuthorizedUser,
    model::user::{
        CreateUserRequest, UpdateUserPasswordRequest, UpdateUserPasswordRequestWithUserId,
        UpdateUserRoleRequest, UpdateUserRoleRequestWithUserId, UserResponse, UsersResponse,
    },
};

#[cfg_attr(debug_assertions, utoipa::path(post, path = "/api/v1/users"))]
#[tracing::instrument(skip(user, registry, req), fields(user_id = %user.user.id.to_string()))]
pub async fn register_user(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }

    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;
    Ok(Json(registered_user.into()))
}

#[cfg_attr(debug_assertions, utoipa::path(get, path = "/api/v1/users"))]
#[tracing::instrument(skip(_user, registry), fields(user_id = %_user.user.id.to_string()))]
pub async fn list_users(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<UsersResponse>> {
    let items = registry
        .user_repository()
        .find_all()
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(UsersResponse { items }))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        delete,
        path = "/api/v1/users/{user_id}",
        params(
            ("user_id" = String, Path, description = "ユーザーID")
        )
    )
)]
#[tracing::instrument(skip(user, registry), fields(user_id = %user.user.id.to_string()))]
pub async fn delete_user(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }

    registry
        .user_repository()
        .delete(DeleteUser { user_id })
        .await?;

    Ok(StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        put,
        path = "/api/v1/users/{user_id}/role",
        params(
            ("user_id" = String, Path, description = "ユーザーID")
        )
    )
)]
#[tracing::instrument(skip(user, registry, req), fields(user_id = %user.user.id.to_string()))]
pub async fn change_role(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> AppResult<StatusCode> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation);
    }

    registry
        .user_repository()
        .update_role(UpdateUserRoleRequestWithUserId::new(user_id, req).into())
        .await?;

    Ok(StatusCode::OK)
}

#[cfg_attr(debug_assertions, utoipa::path(get, path = "/api/v1/users/me"))]
#[tracing::instrument(skip(user), fields(user_id = %user.user.id.to_string()))]
pub async fn get_current_user(user: AuthorizedUser) -> Json<UserResponse> {
    Json(UserResponse::from(user.user))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(put, path = "/api/v1/users/me/password")
)]
#[tracing::instrument(skip(user, registry, req), fields(user_id = %user.user.id.to_string()))]
pub async fn change_password(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserPasswordRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_password(UpdateUserPasswordRequestWithUserId::new(user.id(), req).into())
        .await?;
    Ok(StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path = "/api/v1/users/me/checkouts")
)]
#[tracing::instrument(skip(user, registry), fields(user_id = %user.user.id.to_string()))]
pub async fn get_checkouts(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_by_user_id(user.id())
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}
