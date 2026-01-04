use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{User, CreateUser, Comunicacion, CreateComunicacion};
use std::sync::{Arc, RwLock};
use chrono::Utc;
use crate::auth::Claims; // Importamos Claims
use crate::models::{LoginRequest, LoginResponse};
use crate::auth::create_jwt;

// Definimos el tipo para nuestro estado global (Base de datos en memoria)
pub type AppState = Arc<RwLock<Db>>;

pub struct Db {
    pub users: Vec<User>,
    pub comunicaciones: Vec<Comunicacion>,
    pub next_user_id: i32,
}

// --- Handlers de Usuarios ---

#[utoipa::path(
    get, 
    path = "/users", 
    responses((status = 200, body = [User]), (status = 401, description = "No autorizado")),
    security(("bearer_auth" = [])) // Esto es para Swagger
)]
#[axum::debug_handler]
pub async fn list_users(
    _claims: Claims,
    State(state): State<AppState>
) -> Json<Vec<User>> {
    let users = {
        let db = state.read().unwrap();
        db.users.clone()
    };
    Json(users)
}

#[utoipa::path(post, 
    path = "/users", 
    request_body = CreateUser,
    responses((status = 201, body = [User]), (status = 401, description = "No autorizado")),
    security(("bearer_auth" = [])) // Esto es para Swagger
)]
#[axum::debug_handler]
pub async fn create_user(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>
) -> (StatusCode, Json<User>) {
    let user = {
        let mut db = state.write().unwrap();
        let user = User {
            id: db.next_user_id,
            nombre: payload.nombre,
            apellidos: payload.apellidos,
            telefono: payload.telefono,
            direccion: payload.direccion,
        };
        db.next_user_id += 1;
        db.users.push(user.clone());
        user
    };
    (StatusCode::CREATED, Json(user))
}

#[utoipa::path(delete,
    path = "/users/{id}",
    responses((status = 204), (status = 401, description = "No autorizado"), (status = 404)),
    security(("bearer_auth" = [])) // Esto es para Swagger
)]
#[axum::debug_handler]
pub async fn delete_user(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>
) -> StatusCode {
    let removed = {
        let mut db = state.write().unwrap();
        if let Some(pos) = db.users.iter().position(|u| u.id == id) {
            db.users.remove(pos);
            true
        } else {
            false
        }
    };
    if removed { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}
#[utoipa::path(get,
    path = "/comunicaciones",
    responses((status = 200, body = [Comunicacion]), (status = 401, description = "No autorizado")),
    security(("bearer_auth" = [])) // Esto es para Swagger
)]
#[axum::debug_handler]
pub async fn list_comunicaciones(
    _claims: Claims,
    State(state): State<AppState>
) -> Json<Vec<Comunicacion>> {
    let comunicaciones = {
        let db = state.read().unwrap();
        db.comunicaciones.clone()
    };
    Json(comunicaciones)
}

#[utoipa::path(post, 
    path = "/comunicaciones", 
    request_body = CreateComunicacion, 
    responses(
        (status = 201, body = Comunicacion),
        (status = 400, description = "Usuario no existe"),
        (status = 401, description = "No autorizado")
    ),
    security(("bearer_auth" = [])) // Esto es para Swagger  
)]
#[axum::debug_handler]
pub async fn create_comunicacion(
    _claims: Claims,
    State(state): State<AppState>, 
    Json(payload): Json<CreateComunicacion>
) -> Result<(StatusCode, Json<Comunicacion>), StatusCode> {
    let nueva_com = {
        let mut db = state.write().unwrap();
        
        // Validar si el usuario existe
        if !db.users.iter().any(|u| u.id == payload.usuario_id) {
            return Err(StatusCode::BAD_REQUEST);
        }

        let nueva = Comunicacion {
            id: (db.comunicaciones.len() as i32) + 1,
            fecha: Utc::now().to_rfc3339(),
            tipo: payload.tipo,
            usuario_id: payload.usuario_id,
            resumen: payload.resumen,
        };

        db.comunicaciones.push(nueva.clone());
        nueva
    };

    Ok((StatusCode::CREATED, Json(nueva_com)))
}

#[utoipa::path(delete, 
    path = "/comunicaciones/{id}", 
    responses((status = 204), (status = 404), (status = 401, description = "No autorizado")),
    security(("bearer_auth" = [])) // Esto es para Swagger
)]
#[axum::debug_handler]
pub async fn delete_comunicacion(
    _claims: Claims,
    State(state): State<AppState>, 
    Path(id): Path<i32>
) -> StatusCode {
    let removed = {
        let mut db = state.write().unwrap();
        if let Some(pos) = db.comunicaciones.iter().position(|c| c.id == id) {
            db.comunicaciones.remove(pos);
            true
        } else {
            false
        }
    };
    if removed { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}
// --- Handler de Login para obtener el JWT
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = LoginResponse),
        (status = 401, description = "Credenciales incorrectas")
    )
)]
pub async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    // Validación de ejemplo (Hardcoded)
    if payload.usuario == "admin" && payload.clave == "d3d1c4fc3Aa!" {
        let token = create_jwt(&payload.usuario)?;
        Ok(Json(LoginResponse { token }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}