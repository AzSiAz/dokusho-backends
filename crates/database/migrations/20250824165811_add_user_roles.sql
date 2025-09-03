-- Create user_role enum type
CREATE TYPE user_role AS ENUM ('user', 'admin');

-- Add role column to users table
ALTER TABLE "user"
ADD COLUMN role user_role NOT NULL DEFAULT 'user';

-- Add index for role queries
CREATE INDEX IF NOT EXISTS idx_users_role ON "user"(role);