use kernel::model::id::UserId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[cfg_attr(debug_assertions, derive(ToSchema))]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[cfg_attr(debug_assertions, derive(ToSchema))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessTokenResponse {
    #[cfg_attr(debug_assertions, schema(value_type = String, format = Uuid))]
    pub user_id: UserId,
    pub access_token: String,
}
