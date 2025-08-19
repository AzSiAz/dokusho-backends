use sqlx::PgPool;

use crate::{DatabaseError, models::AuthState};

pub struct AuthStateRepository {
    pool: PgPool,
}

impl AuthStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        state: String,
        redirect_uri: String,
        nonce: String,
        pkce_verifier: Option<String>,
    ) -> Result<AuthState, DatabaseError> {
        let auth_state = sqlx::query_as!(
            AuthState,
            r#"
            INSERT INTO auth_states (state, redirect_uri, nonce, pkce_verifier)
            VALUES ($1, $2, $3, $4)
            RETURNING state, redirect_uri, nonce, pkce_verifier, created_at, expires_at
            "#,
            state,
            redirect_uri,
            nonce,
            pkce_verifier
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(auth_state)
    }

    pub async fn find_by_state(&self, state: &str) -> Result<Option<AuthState>, DatabaseError> {
        let auth_state = sqlx::query_as!(
            AuthState,
            r#"
            SELECT state, redirect_uri, nonce, pkce_verifier, created_at, expires_at
            FROM auth_states
            WHERE state = $1 AND expires_at > NOW()
            "#,
            state
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(auth_state)
    }

    pub async fn delete(&self, state: &str) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth_states
            WHERE state = $1
            "#,
            state
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_expired(&self) -> Result<u64, DatabaseError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth_states
            WHERE expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
