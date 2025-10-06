-- Drop auth_state and user_session tables as we're moving to pure OpenID token validation
-- No longer need to track auth states or sessions

DROP TABLE IF EXISTS user_session;
DROP TABLE IF EXISTS auth_state;