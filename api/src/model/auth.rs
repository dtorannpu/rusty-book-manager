use serde::{Deserialize, Serialize};
use kernel::model::id::UserId;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessTokenResponse {
    pub user_id: UserId,
    pub access_token: String,
}