use axum::{extract::{Path, State}, http::StatusCode, Json};
use sqlx::{AnyPool, query, query_as};
use crate::models::{User, CreateUser, Comunicacion, CreateComunicacion, LoginRequest, LoginResponse};
use crate::auth::{create_jwt, Claims};
use bcrypt::verify;
use tracing::{info, error, warn};

// --- Handlers de Usuarios ---
#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(get, path = "/users", responses((status = 200, body = [User])), security(("bearer_auth" = [])))]
pub async fn list_users(_claims: Claims, State(pool): State<AnyPool>) -> Result<Json<Vec<User>>, StatusCode> {
    info!("Obteniendo lista de usuarios");
    let users = sqlx::query_as::<_, User>("SELECT id, nombre, apellidos, telefono, direccion FROM usuarios")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(users))
}
//#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(post, path = "/users", request_body = CreateUser, responses((status = 201, body = User)), security(("bearer_auth" = [])))]
pub async fn create_user(_claims: Claims, State(pool): State<AnyPool>, Json(payload): Json<CreateUser>) -> Result<(StatusCode, Json<User>), StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO usuarios (nombre, apellidos, telefono, direccion) VALUES ($1, $2, $3, $4) RETURNING id, nombre, apellidos, telefono, direccion"
    )
    .bind(payload.nombre)
    .bind(payload.apellidos)
    .bind(payload.telefono)
    .bind(payload.direccion)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}
//#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(delete, path = "/users/{id}", responses((status = 204), (status = 404)), security(("bearer_auth" = [])))]
pub async fn delete_user(_claims: Claims, State(pool): State<AnyPool>, Path(id): Path<i32>) -> StatusCode {
    let result = sqlx::query("DELETE FROM usuarios WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// --- Handlers de Comunicaciones ---
#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(get, path = "/comunicaciones", responses((status = 200, body = [Comunicacion])), security(("bearer_auth" = [])))]
pub async fn list_comunicaciones(_claims: Claims, State(pool): State<AnyPool>) -> Result<Json<Vec<Comunicacion>>, StatusCode> {
    info!("Obteniendo lista de comunicaciones");
    let coms = sqlx::query_as::<_, Comunicacion>("SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha) AS fecha , tipo, usuario_id, resumen FROM comunicaciones")
        .fetch_all(&pool)
        .await
        .map_err(|e| { error!("Error al obtener comunicaciones: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    info!("comunicaciones obtenidas");
    Ok(Json(coms))
}
//#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(post, path = "/comunicaciones", request_body = CreateComunicacion, responses((status = 201, body = Comunicacion)), security(("bearer_auth" = [])))]
pub async fn create_comunicacion(_claims: Claims, State(pool): State<AnyPool>, Json(payload): Json<CreateComunicacion>) -> Result<(StatusCode, Json<Comunicacion>), StatusCode> {
    // Verificamos si el usuario existe antes de insertar (Integridad referencial)
    let user_exists = sqlx::query("SELECT 1 FROM usuarios WHERE id = $1")
        .bind(payload.usuario_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if user_exists.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Intentamos usar RETURNING (funciona en Postgres, puede fallar en algunas builds de SQLite)
    let insert_sql = "INSERT INTO comunicaciones (tipo, usuario_id, resumen) VALUES ($1, $2, $3) RETURNING id, fecha, tipo, usuario_id, resumen";

    match sqlx::query_as::<_, Comunicacion>(insert_sql)
        .bind(&payload.tipo)
        .bind(payload.usuario_id)
        .bind(&payload.resumen)
        .fetch_one(&pool)
        .await
    {
        Ok(com) => return Ok((StatusCode::CREATED, Json(com))),
        Err(e) => {
            // Registramos el error para debugging y hacemos fallback con transacción
            eprintln!("INSERT with RETURNING failed, falling back: {:?}", e);
        }
    }

    // Fallback: adquirir una conexión y usarla para que INSERT y SELECT estén en la misma conexión
    let mut conn = pool.acquire().await.map_err(|err| { eprintln!("Acquire conn failed: {:?}", err); StatusCode::INTERNAL_SERVER_ERROR })?;

    sqlx::query("INSERT INTO comunicaciones (tipo, usuario_id, resumen) VALUES ($1, $2, $3)")
        .bind(&payload.tipo)
        .bind(payload.usuario_id)
        .bind(&payload.resumen)
        .execute(&mut *conn)
        .await
        .map_err(|err| { eprintln!("Fallback insert failed: {:?}", err); StatusCode::INTERNAL_SERVER_ERROR })?;

    // Intentamos obtener la fila insertada usando la función SQLite primero
    let com = match sqlx::query_as::<_, Comunicacion>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha) AS fecha , tipo, usuario_id, resumen FROM comunicaciones WHERE id = last_insert_rowid()"
    ).fetch_one(&mut *conn).await {
        Ok(c) => c,
        Err(_) => {
            // Como respaldo, obtenemos la última fila por id
            match sqlx::query_as::<_, Comunicacion>("SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha) AS fecha , tipo, usuario_id, resumen FROM comunicaciones ORDER BY id DESC LIMIT 1").fetch_one(&mut *conn).await {
                Ok(c2) => c2,
                Err(err) => { eprintln!("Fetch inserted row failed: {:?}", err); return Err(StatusCode::INTERNAL_SERVER_ERROR); }
            }
        }
    };

    Ok((StatusCode::CREATED, Json(com)))
}
// --- Handler de Login ---

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, description = "Usuario o contraseña incorrectos")
    )
)]
pub async fn login(
    State(pool): State<AnyPool>,
    Json(payload): Json<LoginRequest>
) -> Result<Json<LoginResponse>, StatusCode> {
    // 1. Buscar el hash en la DB
    let row: (String,) = sqlx::query_as("SELECT password_hash FROM credenciales WHERE username = $1")
        .bind(&payload.usuario)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 2. Verificar la contraseña
    let is_valid = verify(&payload.clave, &row.0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_valid {
        let token = create_jwt(&payload.usuario)?;
        Ok(Json(LoginResponse { token }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// --- Handler Delete Comunicación ---
#[tracing::instrument(skip(pool))] // Evitamos loguear todo el objeto pool
#[utoipa::path(
    delete, 
    path = "/comunicaciones/{id}", 
    responses(
        (status = 204, description = "Comunicación eliminada"),
        (status = 404, description = "No encontrada")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_comunicacion(
    _claims: Claims, 
    State(pool): State<AnyPool>, 
    Path(id): Path<i32>
) -> StatusCode {
    let result = sqlx::query("DELETE FROM comunicaciones WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}