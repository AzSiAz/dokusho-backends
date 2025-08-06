#[cfg(test)]
mod tests {
    use dokusho_database::{Database, DatabaseError};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    async fn setup_test_db() -> Result<Database, DatabaseError> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://dokusho:dokusho@localhost:5432/dokusho".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        Ok(Database::from_pool(pool))
    }

    #[tokio::test]
    async fn test_user_crud() -> Result<(), DatabaseError> {
        let db = setup_test_db().await?;

        // Create a user
        let user = db
            .users()
            .create_or_update_user(
                format!("test_sub_{}", Uuid::new_v4()),
                Some("test@example.com".to_string()),
                Some("Test User".to_string()),
            )
            .await?;

        assert!(!user.sub.is_empty());
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.name, Some("Test User".to_string()));

        // Find by ID
        let found_user = db.users().find_by_id(user.id).await?;
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().id, user.id);

        // Find by sub
        let found_by_sub = db.users().find_by_sub(&user.sub).await?;
        assert!(found_by_sub.is_some());
        assert_eq!(found_by_sub.unwrap().id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_auth_state() -> Result<(), DatabaseError> {
        let db = setup_test_db().await?;

        let state = format!("state_{}", Uuid::new_v4());
        let redirect_uri = "http://localhost:3000/callback".to_string();
        let nonce = format!("nonce_{}", Uuid::new_v4());

        // Create auth state
        let auth_state = db
            .auth_states()
            .create(state.clone(), redirect_uri.clone(), nonce.clone())
            .await?;

        assert_eq!(auth_state.state, state);
        assert_eq!(auth_state.redirect_uri, redirect_uri);
        assert_eq!(auth_state.nonce, nonce);

        // Find by state
        let found = db.auth_states().find_by_state(&state).await?;
        assert!(found.is_some());

        // Delete
        let deleted = db.auth_states().delete(&state).await?;
        assert!(deleted);

        // Verify deleted
        let not_found = db.auth_states().find_by_state(&state).await?;
        assert!(not_found.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_operations() -> Result<(), DatabaseError> {
        let db = setup_test_db().await?;

        let source_id = "test_source".to_string();
        let page = 1;
        let data = serde_json::json!({
            "test": "data",
            "items": [1, 2, 3]
        });

        // Set cache
        db.cache()
            .set_popular_series(source_id.clone(), page, data.clone())
            .await?;

        // Get cache
        let cached = db.cache().get_popular_series(&source_id, page).await?;

        assert!(cached.is_some());
        let cached_data = cached.unwrap();
        assert_eq!(cached_data.source_id, source_id);
        assert_eq!(cached_data.page, page);
        assert_eq!(cached_data.data, data);

        // Clear cache
        let deleted_count = db.cache().clear_cache(Some(&source_id)).await?;
        assert!(deleted_count > 0);

        Ok(())
    }
}
