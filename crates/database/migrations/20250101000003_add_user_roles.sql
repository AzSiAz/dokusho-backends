-- Create role enum type
CREATE TYPE user_role AS ENUM ('user', 'admin');

-- Add role column to users table
ALTER TABLE users 
ADD COLUMN role user_role NOT NULL DEFAULT 'user';

-- Add index for role queries
CREATE INDEX idx_users_role ON users(role);