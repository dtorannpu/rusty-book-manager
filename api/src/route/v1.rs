use crate::route::book::build_book_routers;
use crate::route::health::build_health_check_routes;
use crate::route::user::build_user_router;
use axum::Router;
use registry::AppRegistry;

pub fn routes() -> Router<AppRegistry> {
    let router = Router::new()
        .merge(build_health_check_routes())
        .merge(build_book_routers())
        .merge(build_user_router());

    Router::new().nest("/api/v1", router)
}
