use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    DatabaseError,
    models::user::{User, UserPreference, UserRole, UserSession},
};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn is_session_expired(session: &UserSession) -> bool {
        Utc::now() > session.expires_at
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
            INSERT INTO "user" (sub, email, name, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sub) 
            DO UPDATE SET 
                email = COALESCE($2, "user".email),
                name = COALESCE($3, "user".name),
                role = $4,
                updated_at = $6
            RETURNING 
                id, 
                sub, 
                email, 
                name, 
                created_at, 
                updated_at, 
                role as "role: UserRole"
            "#,
            sub,
            email,
            name,
            role as UserRole,
            Utc::now(),
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, 
                sub, 
                email, 
                name, 
                created_at, 
                updated_at, 
                role as "role: UserRole"
            FROM "user" 
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
            SELECT 
                id, 
                sub, 
                email, 
                name, 
                created_at, 
                updated_at, 
                role as "role: UserRole"
            FROM "user" 
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
            SELECT 
                id, 
                sub, 
                email, 
                name, 
                created_at, 
                updated_at, 
                role as "role: UserRole"
            FROM "user"
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
            INSERT INTO user_session (user_id, token_hash, expires_at, created_at, last_used_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, token_hash, expires_at, created_at, last_used_at
            "#,
            user_id,
            token_hash,
            expires_at,
            Utc::now(),
            Utc::now()
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
            FROM user_session
            WHERE token_hash = $1
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn update_session_last_used(
        &self,
        session_id: Uuid,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            UPDATE user_session 
            SET last_used_at = $1
            WHERE id = $2
            "#,
            Utc::now(),
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            DELETE FROM user_session 
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_expired_sessions(&self) -> Result<u64, DatabaseError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_session 
            WHERE expires_at < $1
            "#,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_or_create_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<UserPreference, DatabaseError> {
        // First try to insert with defaults, ignoring if already exists
        sqlx::query!(
            r#"
            INSERT INTO user_preference (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;
        
        // Then fetch the preferences (which now definitely exist)
        let pref = sqlx::query_as!(
            UserPreference,
            r#"
            SELECT user_id, preferred_language, theme, notifications_enabled, created_at, updated_at
            FROM user_preference 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(pref)
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        language: Option<String>,
        theme: Option<String>,
        notifications: Option<bool>,
    ) -> Result<UserPreference, DatabaseError> {
        let mut txn = self.pool.begin().await?;

        // First ensure preferences exist
        sqlx::query!(
            r#"
            INSERT INTO user_preference (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING
            "#,
            user_id
        )
        .execute(&mut *txn)
        .await?;

        // Then update with provided values
        let pref = sqlx::query_as!(
            UserPreference,
            r#"
            UPDATE user_preference 
            SET 
                preferred_language = COALESCE($2, preferred_language),
                theme = COALESCE($3, theme),
                notifications_enabled = COALESCE($4, notifications_enabled),
                updated_at = $5
            WHERE user_id = $1
            RETURNING user_id, preferred_language, theme, notifications_enabled, created_at, updated_at
            "#,
            user_id,
            language,
            theme,
            notifications,
            Utc::now()
        )
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;
        Ok(pref)
    }
}