use crate::model::id::UserId;
use crate::model::user::event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole};
use crate::model::user::User;
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn create(&self, event: CreateUser) -> AppResult<User>;
    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()>;
    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()>;
    async fn delete(&self, event: DeleteUser) -> AppResult<()>;
}
