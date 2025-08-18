use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};

use crate::{errors::AuthError, models::Claims};

pub fn generate_jwt(claims: &Claims, secret: &str) -> Result<String, AuthError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AuthError::from)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(AuthError::from)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    general_purpose::URL_SAFE_NO_PAD.encode(result)
}

pub fn generate_random_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn generate_state_token() -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes())
}

pub fn generate_nonce() -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_jwt_generation_and_validation() {
        let claims = Claims::new(
            Uuid::new_v4(),
            "test_sub".to_string(),
            Some("test@example.com".to_string()),
            Some("Test User".to_string()),
            dokusho_database::models::UserRole::User,
            24,
        );

        let secret = "test_secret";
        let token = generate_jwt(&claims, secret).unwrap();

        let validated_claims = validate_jwt(&token, secret).unwrap();
        assert_eq!(validated_claims.sub, claims.sub);
        assert_eq!(validated_claims.email, claims.email);
        assert_eq!(validated_claims.name, claims.name);
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, token);
    }

    #[test]
    fn test_random_token_generation() {
        let token1 = generate_random_token();
        let token2 = generate_random_token();

        assert_ne!(token1, token2);
    }
}
