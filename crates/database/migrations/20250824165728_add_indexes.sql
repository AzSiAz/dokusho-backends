-- Auth states indexes
CREATE INDEX IF NOT EXISTS idx_auth_states_expires ON auth_state(expires_at);
CREATE INDEX IF NOT EXISTS idx_auth_states_created ON auth_state(created_at);

-- Users indexes
CREATE INDEX IF NOT EXISTS idx_users_sub ON "user"(sub);
CREATE INDEX IF NOT EXISTS idx_users_email ON "user"(email);
CREATE INDEX IF NOT EXISTS idx_users_created ON "user"(created_at);

-- User sessions indexes
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_session(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_session(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_session(token_hash);