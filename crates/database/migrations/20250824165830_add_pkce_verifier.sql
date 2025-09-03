-- Add PKCE verifier to auth_states table
ALTER TABLE auth_state
ADD COLUMN pkce_verifier TEXT;