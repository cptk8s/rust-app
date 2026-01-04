use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{User, CreateUser, Comunicacion, CreateComunicacion};
use std::sync::{Arc, RwLock};
use chrono::Utc;
use crate::auth::Claims; // Importamos Claims

// Definimos el tipo para nuestro estado global (Base de datos en memoria)
pub type AppState = Arc<RwLock<Db>>;

pub struct Db {
    pub users: Vec<User>,
    pub comunicaciones: Vec<Comunicacion>,
    pub next_user_id: i32,
}

// --- Handlers de Usuarios ---


#[utoipa::path(get, path = "/users", responses((status = 200, body = [User])))]
pub async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let db = state.read().unwrap();
    Json(db.users.clone())
}

#[utoipa::path(post, path = "/users", request_body = CreateUser, responses((status = 201, body = User)))]
pub async fn create_user(State(state): State<AppState>, Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
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
    (StatusCode::CREATED, Json(user))
}

#[utoipa::path(delete, path = "/users/{id}", responses((status = 204), (status = 404)))]
pub async fn delete_user(State(state): State<AppState>, Path(id): Path<i32>) -> StatusCode {
    let mut db = state.write().unwrap();
    if let Some(pos) = db.users.iter().position(|u| u.id == id) {
        db.users.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
#[utoipa::path(get, path = "/comunicaciones", responses((status = 200, body = [Comunicacion])))]
pub async fn list_comunicaciones(State(state): State<AppState>) -> Json<Vec<Comunicacion>> {
    let db = state.read().unwrap();
    Json(db.comunicaciones.clone())
}

#[utoipa::path(post, path = "/comunicaciones", request_body = CreateComunicacion, 
    responses(
        (status = 201, body = Comunicacion),
        (status = 400, description = "Usuario no existe")
    )
)]
pub async fn create_comunicacion(
    State(state): State<AppState>, 
    Json(payload): Json<CreateComunicacion>
) -> Result<(StatusCode, Json<Comunicacion>), StatusCode> {
    let mut db = state.write().unwrap();
    
    // Validar si el usuario existe
    if !db.users.iter().any(|u| u.id == payload.usuario_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let nueva_com = Comunicacion {
        id: (db.comunicaciones.len() as i32) + 1,
        fecha: Utc::now().to_rfc3339(),
        tipo: payload.tipo,
        usuario_id: payload.usuario_id,
        resumen: payload.resumen,
    };

    db.comunicaciones.push(nueva_com.clone());
    Ok((StatusCode::CREATED, Json(nueva_com)))
}

#[utoipa::path(delete, path = "/comunicaciones/{id}", responses((status = 204), (status = 404)))]
pub async fn delete_comunicacion(State(state): State<AppState>, Path(id): Path<i32>) -> StatusCode {
    let mut db = state.write().unwrap();
    if let Some(pos) = db.comunicaciones.iter().position(|c| c.id == id) {
        db.comunicaciones.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}