-- Add PKCE verifier to auth_states table
ALTER TABLE auth_states 
ADD COLUMN pkce_verifier TEXT;