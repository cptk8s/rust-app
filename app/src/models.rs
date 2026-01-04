use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct User {
    pub id: i32,
    pub nombre: String,
    pub apellidos: String,
    pub telefono: String,
    pub direccion: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub nombre: String,
    pub apellidos: String,
    pub telefono: String,
    pub direccion: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Comunicacion {
    pub id: i32,
    pub fecha: String,
    pub tipo: String,
    pub usuario_id: i32,
    pub resumen: String,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateComunicacion {
    pub tipo: String,
    pub usuario_id: i32,
    pub resumen: String,
}