mod models;
mod handlers;
mod openapi;
mod auth;

use axum::{routing::{get, post, delete}, Router};
use std::net::SocketAddr;
use std::env;
use dotenvy::dotenv;
use sqlx::{any::install_default_drivers, AnyPool};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // 1. Cargar variables de entorno
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "api_comunicaciones=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Configurar conexión a la DB (Dinámica)
    let (pool, is_sqlite) = setup_database().await;

    // 3. Crear tablas si no existen
    init_db(&pool, is_sqlite).await;    

    // 4. Configurar rutas
    let app = Router::new()
        .route("/login", post(handlers::login))
        .route("/users", get(handlers::list_users).post(handlers::create_user))
        .route("/users/:id", delete(handlers::delete_user))
        .route("/comunicaciones", get(handlers::list_comunicaciones).post(handlers::create_comunicacion))
        .route("/comunicaciones/:id", delete(handlers::delete_comunicacion))
        .with_state(pool) // Pasamos el pool de conexión a los handlers
        .layer(TraceLayer::new_for_http())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Servidor en http://{}", addr);
    println!("📖 Swagger en http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn is_sqlite_conn(connection_string: &str) -> bool {
    connection_string.starts_with("sqlite")
}

async fn setup_database() -> (AnyPool, bool) {
    let db_url = env::var("DB_URL").unwrap_or_default();
    let db_usr = env::var("DB_USR").unwrap_or_default();
    let db_pwd = env::var("DB_PWD").unwrap_or_default();

    let connection_string = if !db_url.is_empty() {
        format!("postgres://{}:{}@{}/postgres", db_usr, db_pwd, db_url)
    } else {
        "sqlite://database.db?mode=rwc".to_string()
    };

    // Detectamos si la conexión es SQLite basándonos en la cadena
    let is_sqlite = is_sqlite_conn(&connection_string);

    install_default_drivers();
    let pool = AnyPool::connect(&connection_string).await.expect("Error al conectar a la DB");
    (pool, is_sqlite)
} 

async fn init_db(pool: &AnyPool, is_sqlite: bool) {
    // Definimos el esquema. Usamos sintaxis estándar compatible.
    // Nota: 'AUTOINCREMENT' es de SQLite, en Postgres se usa 'SERIAL'. 
    // Para que sea compatible con AnyPool, usamos un truco:

    let create_usuarios = if is_sqlite {
        "CREATE TABLE IF NOT EXISTS usuarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            apellidos TEXT NOT NULL,
            telefono TEXT NOT NULL,
            direccion TEXT NOT NULL
        );"
    } else {
        "CREATE TABLE IF NOT EXISTS usuarios (
            id SERIAL PRIMARY KEY,
            nombre TEXT NOT NULL,
            apellidos TEXT NOT NULL,
            telefono TEXT NOT NULL,
            direccion TEXT NOT NULL
        );"
    };

    let create_comunicaciones = if is_sqlite {
        "CREATE TABLE IF NOT EXISTS comunicaciones (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            tipo TEXT NOT NULL,
            usuario_id INTEGER NOT NULL,
            resumen TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );"
    } else {
        "CREATE TABLE IF NOT EXISTS comunicaciones (
            id SERIAL PRIMARY KEY,
            fecha TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            tipo TEXT NOT NULL,
            usuario_id INTEGER NOT NULL,
            resumen TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );"
    };

    sqlx::query(create_usuarios).execute(pool).await.expect("Error creando tabla usuarios");
    sqlx::query(create_comunicaciones).execute(pool).await.expect("Error creando tabla comunicaciones");
    
    println!("✅ Base de datos verificada/inicializada correctamente.");

    let create_credenciales = if is_sqlite {
        "CREATE TABLE IF NOT EXISTS credenciales (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            usuario_id INTEGER NOT NULL,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );"
    } else {
        "CREATE TABLE IF NOT EXISTS credenciales (
            id SERIAL PRIMARY KEY,
            usuario_id INTEGER NOT NULL,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );"
    };

    sqlx::query(create_credenciales).execute(pool).await.expect("Error creando credenciales");

    // Opcional: Crear usuario admin por defecto (password: admin123)
    let has_admin = sqlx::query("SELECT 1 FROM credenciales WHERE username = 'admin'")
        .fetch_optional(pool).await.unwrap();
    
    if has_admin.is_none() {
        let hashed = bcrypt::hash("d3d1c4fc3Aa", bcrypt::DEFAULT_COST).unwrap();
        // Primero creamos el usuario 'Sistema' para ligar la credencial
        let u_id: (i32,) = sqlx::query_as("INSERT INTO usuarios (nombre, apellidos, telefono, direccion) VALUES ('Admin', 'Sistema', '000', 'N/A') RETURNING id")
            .fetch_one(pool).await.unwrap();
        
        sqlx::query("INSERT INTO credenciales (usuario_id, username, password_hash) VALUES ($1, 'admin', $2)")
            .bind(u_id.0).bind(hashed).execute(pool).await.unwrap();
        println!("👤 Usuario administrador creado por defecto: admin / d3d1c4fc3Aa");
    }

}

#[cfg(test)]
mod tests {
    use super::is_sqlite_conn;

    #[test]
    fn detects_sqlite() {
        assert!(is_sqlite_conn("sqlite://database.db"));
        assert!(is_sqlite_conn("sqlite:///tmp/db.sqlite"));
    }

    #[test]
    fn detects_non_sqlite() {
        assert!(!is_sqlite_conn("postgres://user:pass@localhost/postgres"));
        assert!(!is_sqlite_conn("mysql://root@localhost/db"));
    }
}
