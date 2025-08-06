-- Performance indexes

-- Auth states
CREATE INDEX IF NOT EXISTS idx_auth_states_expires ON auth_states(expires_at);
CREATE INDEX IF NOT EXISTS idx_auth_states_created ON auth_states(created_at);

-- Users
CREATE INDEX IF NOT EXISTS idx_users_sub ON users(sub);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created ON users(created_at);

-- User sessions
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions(token_hash);

-- Cache tables
CREATE INDEX IF NOT EXISTS idx_popular_series_cache_updated ON popular_series_cache(updated_at);
CREATE INDEX IF NOT EXISTS idx_latest_series_cache_updated ON latest_series_cache(updated_at);
CREATE INDEX IF NOT EXISTS idx_series_detail_cache_updated ON series_detail_cache(updated_at);

-- Workflows
CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
CREATE INDEX IF NOT EXISTS idx_workflows_created ON workflows(created_at);
CREATE INDEX IF NOT EXISTS idx_workflows_updated ON workflows(updated_at);