use chrono::{FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{
    DatabaseError,
    entities::{prelude::*, sea_orm_active_enums::UserRole, user, user_preference, user_session},
};

pub struct UserRepository {
    conn: DatabaseConnection,
}

impl UserRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub fn is_session_expired(session: &user_session::Model) -> bool {
        Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()) > session.expires_at
    }

    pub async fn create_or_update_user(
        &self,
        sub: String,
        email: Option<String>,
        name: Option<String>,
        role: UserRole,
    ) -> Result<user::Model, DatabaseError> {
        // First try to find existing user
        let existing = User::find()
            .filter(user::Column::Sub.eq(&sub))
            .one(&self.conn)
            .await?;

        let user_entity = if let Some(existing_user) = existing {
            // Update existing user
            let mut active_model: user::ActiveModel = existing_user.into();
            if email.is_some() {
                active_model.email = Set(email.clone());
            }
            if name.is_some() {
                active_model.name = Set(name.clone());
            }
            active_model.role = Set(role);
            active_model.updated_at = Set(Some(Utc::now().into()));
            active_model.update(&self.conn).await?
        } else {
            // Create new user
            let new_user = user::ActiveModel {
                id: Set(Uuid::new_v4()),
                sub: Set(sub.clone()),
                email: Set(email.clone()),
                name: Set(name.clone()),
                role: Set(role),
                created_at: Set(Some(Utc::now().into())),
                updated_at: Set(Some(Utc::now().into())),
            };
            new_user.insert(&self.conn).await?
        };

        Ok(user_entity)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<user::Model>, DatabaseError> {
        let user = User::find_by_id(id).one(&self.conn).await?;

        Ok(user)
    }

    pub async fn find_by_sub(&self, sub: &str) -> Result<Option<user::Model>, DatabaseError> {
        let user = User::find()
            .filter(user::Column::Sub.eq(sub))
            .one(&self.conn)
            .await?;

        Ok(user)
    }

    pub async fn find_all(&self) -> Result<Vec<user::Model>, DatabaseError> {
        let users = User::find().all(&self.conn).await?;

        Ok(users)
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expiry_hours: i64,
    ) -> Result<user_session::Model, DatabaseError> {
        let expires_at = (Utc::now() + chrono::Duration::hours(expiry_hours)).into();

        let session = user_session::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            created_at: Set(Some(Utc::now().into())),
            last_used_at: Set(Some(Utc::now().into())),
        };

        let session_entity = session.insert(&self.conn).await?;

        Ok(session_entity)
    }

    pub async fn find_session_by_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<user_session::Model>, DatabaseError> {
        let session = UserSession::find()
            .filter(user_session::Column::TokenHash.eq(token_hash))
            .filter(user_session::Column::ExpiresAt.gt(Utc::now()))
            .one(&self.conn)
            .await?;

        Ok(session)
    }

    pub async fn update_session_last_used(&self, session_id: Uuid) -> Result<(), DatabaseError> {
        let session = UserSession::find_by_id(session_id).one(&self.conn).await?;

        if let Some(session) = session {
            let mut active_model: user_session::ActiveModel = session.into();
            active_model.last_used_at = Set(Some(Utc::now().into()));
            active_model.update(&self.conn).await?;
        }

        Ok(())
    }

    pub async fn delete_session(&self, session_id: Uuid) -> Result<bool, DatabaseError> {
        let result = UserSession::delete_by_id(session_id)
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected > 0)
    }

    /// Atomically rotate a session: delete the old one and create a new one
    /// This ensures that token refresh is atomic and prevents race conditions
    pub async fn rotate_session(
        &self,
        user_id: Uuid,
        old_token_hash: &str,
        new_token_hash: String,
        expiry_hours: i64,
    ) -> Result<user_session::Model, DatabaseError> {
        let expires_at = (Utc::now() + chrono::Duration::hours(expiry_hours)).into();

        // Start a transaction
        let txn = self.conn.begin().await?;

        // Delete the old session
        UserSession::delete_many()
            .filter(user_session::Column::UserId.eq(user_id))
            .filter(user_session::Column::TokenHash.eq(old_token_hash))
            .exec(&txn)
            .await?;

        // Create the new session
        let session = user_session::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            token_hash: Set(new_token_hash),
            expires_at: Set(expires_at),
            created_at: Set(Some(Utc::now().into())),
            last_used_at: Set(Some(Utc::now().into())),
        };

        let session_entity = session.insert(&txn).await?;

        // Commit the transaction
        txn.commit().await?;

        Ok(session_entity)
    }

    pub async fn delete_expired_sessions(&self) -> Result<u64, DatabaseError> {
        let result = UserSession::delete_many()
            .filter(user_session::Column::ExpiresAt.lt(Utc::now()))
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected)
    }

    pub async fn get_or_create_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<user_preference::Model, DatabaseError> {
        // Try to find existing preferences
        let existing = UserPreference::find_by_id(user_id).one(&self.conn).await?;

        let prefs_entity = if let Some(prefs) = existing {
            // Update timestamp
            let mut active_model: user_preference::ActiveModel = prefs.into();
            active_model.updated_at = Set(Some(Utc::now().into()));
            active_model.update(&self.conn).await?
        } else {
            // Create new preferences
            let new_prefs = user_preference::ActiveModel {
                user_id: Set(user_id),
                preferred_language: Set(Some("en".to_string())),
                theme: Set(Some("light".to_string())),
                notifications_enabled: Set(Some(true)),
                created_at: Set(Some(Utc::now().into())),
                updated_at: Set(Some(Utc::now().into())),
            };
            new_prefs.insert(&self.conn).await?
        };

        Ok(prefs_entity)
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        preferred_language: Option<String>,
        theme: Option<String>,
        notifications_enabled: Option<bool>,
    ) -> Result<user_preference::Model, DatabaseError> {
        let prefs = UserPreference::find_by_id(user_id)
            .one(&self.conn)
            .await?
            .ok_or(DatabaseError::NotFound)?;

        let mut active_model: user_preference::ActiveModel = prefs.into();

        if preferred_language.is_some() {
            active_model.preferred_language = Set(preferred_language);
        }
        if theme.is_some() {
            active_model.theme = Set(theme);
        }
        if notifications_enabled.is_some() {
            active_model.notifications_enabled = Set(notifications_enabled);
        }
        active_model.updated_at = Set(Some(Utc::now().into()));

        let updated_prefs = active_model.update(&self.conn).await?;

        Ok(updated_prefs)
    }
}
