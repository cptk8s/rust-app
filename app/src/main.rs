mod models;
mod handlers;
mod auth;
mod openapi;

use axum::{routing::{get, post, delete}, Router};
use std::sync::{Arc, RwLock};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Inicializamos nuestra "Base de datos" en memoria
    let shared_state = Arc::new(RwLock::new(handlers::Db {
        users: Vec::new(),
        comunicaciones: Vec::new(),
        next_user_id: 1,
    }));

    let app = Router::new()
    .route("/login", post(handlers::login)) // Pública
    //Rutas de usuario
    .route("/users", get(handlers::list_users).post(handlers::create_user))
    .route("/users/:id", delete(handlers::delete_user))
    //Rutas de comunicaciones
    .route("/comunicaciones", get(handlers::list_comunicaciones).post(handlers::create_comunicacion))
    .route("/comunicaciones/:id", delete(handlers::delete_comunicacion))
    .with_state(shared_state)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 CRUD listo en http://127.0.0.1:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}