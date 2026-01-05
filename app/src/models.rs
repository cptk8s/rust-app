use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, sqlx::FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub nombre: String,
    pub apellidos: String,
    pub telefono: String,
    pub direccion: String,
}

#[derive(Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CreateUser {
    pub nombre: String,
    pub apellidos: String,
    pub telefono: String,
    pub direccion: String,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema, sqlx::FromRow, Clone)]
pub struct Comunicacion {
    pub id: i32,
    pub fecha: String,
    pub tipo: String,
    pub usuario_id: i32,
    pub resumen: String,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct CreateComunicacion {
    pub tipo: String,
    pub usuario_id: i32,
    pub resumen: String,
}
#[derive(serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct LoginRequest {
    pub usuario: String,
    pub clave: String,
}

#[derive(serde::Serialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct LoginResponse {
    pub token: String,
}
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow, Clone)]
pub struct Credencial {
    pub id: i32,
    pub usuario_id: i32,
    pub username: String,
    pub password_hash: String,
}