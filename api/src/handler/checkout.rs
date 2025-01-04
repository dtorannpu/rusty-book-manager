use crate::extractor::AuthorizedUser;
use crate::model::checkout::CheckoutsResponse;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use kernel::model::checkout::event::{CreateCheckout, UpdateReturned};
use kernel::model::id::{BookId, CheckoutId};
use registry::AppRegistry;
use shared::error::AppResult;
#[cfg_attr(
    debug_assertions,
    utoipa::path(
        post,
        path = "/api/v1/books/{book_id}/checkouts",
        params(
            ("book_id" = String, description = "蔵書ID")
        )
    )
)]
#[tracing::instrument(
    skip(user, registry),
    fields(user_id = %user.user.id.to_string())
)]
pub async fn checkout_book(
    user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let create_checkout_history = CreateCheckout::new(book_id, user.id(), chrono::Utc::now());

    registry
        .checkout_repository()
        .create(create_checkout_history)
        .await
        .map(|_| StatusCode::OK)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        put,
        path = "/api/v1/books/{book_id}/checkouts/{checkout_id}/returned",
        params(
            ("book_id" = String, description = "蔵書ID"),
            ("checkout_id" = String, description = "チェックアウトID")
        )
    )
)]
#[tracing::instrument(
    skip(user, registry),
    fields(user_id = %user.user.id.to_string())
)]
pub async fn return_book(
    user: AuthorizedUser,
    Path((book_id, checkout_id)): Path<(BookId, CheckoutId)>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let update_returned = UpdateReturned::new(checkout_id, book_id, user.id(), chrono::Utc::now());

    registry
        .checkout_repository()
        .update_returned(update_returned)
        .await
        .map(|_| StatusCode::OK)
}

#[cfg_attr(debug_assertions, utoipa::path(get, path = "/api/v1/books/checkouts"))]
#[tracing::instrument(
    skip(_user, registry),
    fields(user_id = %_user.user.id.to_string())
)]
pub async fn show_checked_out_list(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_all()
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        get,
        path = "/api/v1/books/{book_id}/checkout-history",
        params(
            ("book_id" = String, description = "蔵書ID")
        )
    )
)]
#[tracing::instrument(
    skip(_user, registry),
    fields(user_id = %_user.user.id.to_string())
)]
pub async fn checkout_history(
    _user: AuthorizedUser,
    Path(book_id): Path<BookId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_history_by_book_id(book_id)
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}
