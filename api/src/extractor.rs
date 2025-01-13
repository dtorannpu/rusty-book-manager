use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use kernel::model::auth::AccessToken;
use kernel::model::id::UserId;
use kernel::model::role::Role;
use kernel::model::user::User;
use registry::AppRegistry;
use shared::error::AppError;
use std::future::Future;

pub struct AuthorizedUser {
    pub access_token: AccessToken,
    pub user: User,
}

impl AuthorizedUser {
    pub fn id(&self) -> UserId {
        self.user.id
    }

    pub fn is_admin(&self) -> bool {
        self.user.role == Role::Admin
    }
}

#[async_trait]
impl FromRequestParts<AppRegistry> for AuthorizedUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        registry: &AppRegistry,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            let TypedHeader(Authorization(bearer)) = parts
                .extract::<TypedHeader<Authorization<Bearer>>>()
                .await
                .map_err(|_| AppError::UnauthorizedError)?;
            let access_token = AccessToken(bearer.token().to_string());

            let user_id = registry
                .auth_repository()
                .fetch_user_id_from_token(&access_token)
                .await?
                .ok_or(AppError::UnauthenticatedError)?;

            let user = registry
                .user_repository()
                .find_current_user(user_id)
                .await?
                .ok_or(AppError::UnauthenticatedError)?;

            Ok(Self { access_token, user })
        }
    }
}
