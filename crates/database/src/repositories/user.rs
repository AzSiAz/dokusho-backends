use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{User, UserPreferences, UserRole, UserSession},
    DatabaseError,
};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_or_update_user(
        &self,
        sub: String,
        email: Option<String>,
        name: Option<String>,
        role: UserRole,
    ) -> Result<User, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (sub, email, name, role)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (sub) DO UPDATE
            SET email = COALESCE($2, users.email),
                name = COALESCE($3, users.name),
                role = $4,
                updated_at = NOW()
            RETURNING id, sub, email, name, role as "role: UserRole", created_at, updated_at
            "#,
            sub,
            email,
            name,
            role as UserRole
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, sub, email, name, role as "role: UserRole", created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_sub(&self, sub: &str) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, sub, email, name, role as "role: UserRole", created_at, updated_at
            FROM users
            WHERE sub = $1
            "#,
            sub
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_all(&self) -> Result<Vec<User>, DatabaseError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, sub, email, name, role as "role: UserRole", created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expiry_hours: i64,
    ) -> Result<UserSession, DatabaseError> {
        let expires_at = Utc::now() + chrono::Duration::hours(expiry_hours);

        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, expires_at, created_at, last_used_at
            "#,
            user_id,
            token_hash,
            expires_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn find_session_by_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserSession>, DatabaseError> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, last_used_at
            FROM user_sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn update_session_last_used(&self, session_id: Uuid) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            UPDATE user_sessions
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_session(&self, session_id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_sessions
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Atomically rotate a session: delete the old one and create a new one
    /// This ensures that token refresh is atomic and prevents race conditions
    pub async fn rotate_session(
        &self,
        user_id: Uuid,
        old_token_hash: &str,
        new_token_hash: String,
        expiry_hours: i64,
    ) -> Result<UserSession, DatabaseError> {
        let expires_at = Utc::now() + chrono::Duration::hours(expiry_hours);

        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // Delete the old session
        sqlx::query!(
            r#"
            DELETE FROM user_sessions
            WHERE user_id = $1 AND token_hash = $2
            "#,
            user_id,
            old_token_hash
        )
        .execute(&mut *tx)
        .await?;

        // Create the new session
        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, expires_at, created_at, last_used_at
            "#,
            user_id,
            new_token_hash,
            expires_at
        )
        .fetch_one(&mut *tx)
        .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(session)
    }

    pub async fn delete_expired_sessions(&self) -> Result<u64, DatabaseError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_sessions
            WHERE expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_or_create_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<UserPreferences, DatabaseError> {
        let preferences = sqlx::query_as!(
            UserPreferences,
            r#"
            INSERT INTO user_preferences (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO UPDATE
            SET updated_at = NOW()
            RETURNING user_id, preferred_language, theme, notifications_enabled, created_at, updated_at
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(preferences)
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        preferred_language: Option<String>,
        theme: Option<String>,
        notifications_enabled: Option<bool>,
    ) -> Result<UserPreferences, DatabaseError> {
        let preferences = sqlx::query_as!(
            UserPreferences,
            r#"
            UPDATE user_preferences
            SET preferred_language = COALESCE($2, preferred_language),
                theme = COALESCE($3, theme),
                notifications_enabled = COALESCE($4, notifications_enabled),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING user_id, preferred_language, theme, notifications_enabled, created_at, updated_at
            "#,
            user_id,
            preferred_language,
            theme,
            notifications_enabled
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(preferences)
    }
}
