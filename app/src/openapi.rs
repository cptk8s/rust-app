use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use crate::models::*;
use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    // 1. Registramos todos los paths (rutas) de los handlers
    paths(
        handlers::login,
        handlers::list_users,
        handlers::create_user,
        handlers::delete_user,
        handlers::list_comunicaciones,
        handlers::create_comunicacion,
        handlers::delete_comunicacion
    ),
    // 2. Registramos todos los esquemas de datos (JSON)
    components(
        schemas(
            User, CreateUser, 
            Comunicacion, CreateComunicacion, 
            LoginRequest, LoginResponse
        )
    ),
    // 3. Aplicamos el modificador para el candado de seguridad JWT
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Endpoints de autenticación"),
        (name = "Usuarios", description = "Gestión de usuarios"),
        (name = "Comunicaciones", description = "Gestión de historial de comunicaciones")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
