-- Add migration script here

CREATE TYPE "user_role" AS ENUM ('USER', 'ADMIN');

CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  email TEXT UNIQUE NOT NULL,
  password TEXT NOT NULL,
  role user_role NOT NULL DEFAULT 'USER',
  created_at TIMESTAMP DEFAULT NOW()
);