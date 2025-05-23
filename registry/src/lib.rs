use adapter::database::ConnectionPool;
use adapter::redis::RedisClient;
use adapter::repository::auth::AuthRepositoryImpl;
use adapter::repository::book::BookRepositoryImpl;
use adapter::repository::checkout::CheckoutRepositoryImpl;
use adapter::repository::health::HealthCheckRepositoryImpl;
use adapter::repository::user::UserRepositoryImpl;
use kernel::repository::auth::AuthRepository;
use kernel::repository::book::BookRepository;
use kernel::repository::checkout::CheckoutRepository;
use kernel::repository::health::HealthCheckRepository;
use kernel::repository::user::UserRepository;
use shared::config::AppConfig;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppRegistryImpl {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    book_repository: Arc<dyn BookRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
    checkout_repository: Arc<dyn CheckoutRepository>,
}

impl AppRegistryImpl {
    pub fn new(
        pool: ConnectionPool,
        redis_client: Arc<RedisClient>,
        app_config: AppConfig,
    ) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let book_repository = Arc::new(BookRepositoryImpl::new(pool.clone()));
        let auth_repository = Arc::new(AuthRepositoryImpl::new(
            pool.clone(),
            redis_client.clone(),
            app_config.auth.ttl,
        ));
        let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
        let checkout_repository = Arc::new(CheckoutRepositoryImpl::new(pool.clone()));

        Self {
            health_check_repository,
            book_repository,
            auth_repository,
            user_repository,
            checkout_repository,
        }
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
    fn book_repository(&self) -> Arc<dyn BookRepository>;
    fn auth_repository(&self) -> Arc<dyn AuthRepository>;
    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository>;
    fn user_repository(&self) -> Arc<dyn UserRepository>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    fn book_repository(&self) -> Arc<dyn BookRepository> {
        self.book_repository.clone()
    }

    fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }

    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository> {
        self.checkout_repository.clone()
    }

    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }
}

#[derive(Clone)]
pub struct AppRegistry(pub Arc<dyn AppRegistryExt + Send + Sync + 'static>);

impl From<Arc<dyn AppRegistryExt + Send + Sync + 'static>> for AppRegistry {
    fn from(value: Arc<dyn AppRegistryExt + Send + Sync + 'static>) -> Self {
        Self(value)
    }
}

impl Deref for AppRegistry {
    type Target = Arc<dyn AppRegistryExt + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
