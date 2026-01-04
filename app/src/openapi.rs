use utoipa::OpenApi;
use crate::models::*;
use crate::handlers::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_users, create_user, delete_user,
        list_comunicaciones, create_comunicacion, delete_comunicacion
    ),
    components(schemas(User, CreateUser, Comunicacion, CreateComunicacion))
)]
pub struct ApiDoc;