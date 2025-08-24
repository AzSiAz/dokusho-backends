use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::{
    DatabaseError,
    entities::{auth_state, prelude::*},
};

pub struct AuthStateRepository {
    conn: DatabaseConnection,
}

impl AuthStateRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub fn is_expired(auth_state: &auth_state::Model) -> bool {
        auth_state
            .expires_at
            .is_some_and(|exp| Utc::now().with_timezone(exp.offset()) > exp)
    }

    pub async fn create(
        &self,
        state: String,
        redirect_uri: String,
        nonce: String,
        pkce_verifier: Option<String>,
    ) -> Result<auth_state::Model, DatabaseError> {
        let auth_state = auth_state::ActiveModel {
            state: Set(state),
            redirect_uri: Set(redirect_uri),
            nonce: Set(nonce),
            pkce_verifier: Set(pkce_verifier),
            created_at: Set(Some(Utc::now().into())),
            expires_at: Set(Some((Utc::now() + chrono::Duration::minutes(10)).into())),
        };

        let auth_state_entity = auth_state.insert(&self.conn).await?;

        Ok(auth_state_entity)
    }

    pub async fn find_by_state(
        &self,
        state: &str,
    ) -> Result<Option<auth_state::Model>, DatabaseError> {
        let auth_state = AuthState::find_by_id(state)
            .filter(auth_state::Column::ExpiresAt.gt(Utc::now()))
            .one(&self.conn)
            .await?;

        Ok(auth_state)
    }

    pub async fn delete(&self, state: &str) -> Result<bool, DatabaseError> {
        let result = AuthState::delete_by_id(state).exec(&self.conn).await?;

        Ok(result.rows_affected > 0)
    }

    pub async fn delete_expired(&self) -> Result<u64, DatabaseError> {
        let result = AuthState::delete_many()
            .filter(auth_state::Column::ExpiresAt.lt(Utc::now()))
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected)
    }
}
