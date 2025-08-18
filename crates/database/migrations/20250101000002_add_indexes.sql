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
