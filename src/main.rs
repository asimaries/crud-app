use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use controller::{create_user, delete_user, get_user_by_id, health_check, list_users, update_user};
use dotenv::dotenv;
use user_service::UserService;
mod controller;
mod model;
mod user_service;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let service = UserService::new().await.unwrap();

    let user_router = Router::new()
        .route("/:id", get(get_user_by_id))
        .route("/", post(create_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
        .layer(Extension(service.clone()));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(list_users))
        .nest("/user", user_router)
        .layer(Extension(service.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening");

    let _ = axum::serve(listener, app).await.unwrap();
}
