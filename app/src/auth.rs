use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"tu_clave_secreta_super_segura"; // En producción, usa variables de entorno

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // El nombre de usuario o ID
    pub exp: usize,  // Fecha de expiración
}

// Implementamos FromRequestParts para que actúe como un "filtro" en los handlers
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extraer el header Authorization
        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));

        match auth_header {
            Some(token) => {
                let token_data = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(JWT_SECRET),
                    &Validation::default(),
                ).map_err(|_| StatusCode::UNAUTHORIZED)?;
                
                Ok(token_data.claims)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

// Función auxiliar para generar tokens (para el login)
pub fn create_jwt(username: &str) -> Result<String, StatusCode> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}