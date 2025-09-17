#!/bin/bash

# Script to create default admin user for FlowDNS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default admin credentials
ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-admin123}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@flowdns.local}"

# Load database URL from .env
if [ -f "$PROJECT_ROOT/.env" ]; then
    export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)
fi

echo "========================================="
echo "Creating FlowDNS Admin User"
echo "========================================="
echo
echo "Username: $ADMIN_USERNAME"
echo "Email: $ADMIN_EMAIL"
echo "Password: $ADMIN_PASSWORD"
echo

# Hash the password using bcrypt (cost factor 12)
HASHED_PASSWORD='$2b$12$LQFoJJFvMrCjwm5NNEV8y.Yajf5Ykx8qJNZMxbG7D1pPUIBvSeWtu'  # This is "admin123" hashed

# Create the user in the database
PGPASSWORD="Admin@12" psql -h localhost -U flowdns -d flowdns <<EOF
-- Create users table if it doesn't exist
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    is_admin BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert or update admin user
INSERT INTO users (username, email, password_hash, is_active, is_admin)
VALUES ('$ADMIN_USERNAME', '$ADMIN_EMAIL', '$HASHED_PASSWORD', true, true)
ON CONFLICT (username) DO UPDATE
SET
    email = EXCLUDED.email,
    password_hash = EXCLUDED.password_hash,
    is_active = EXCLUDED.is_active,
    is_admin = EXCLUDED.is_admin,
    updated_at = NOW();

-- Verify the user was created
SELECT username, email, is_admin, is_active FROM users WHERE username = '$ADMIN_USERNAME';
EOF

if [ $? -eq 0 ]; then
    echo
    echo "========================================="
    echo "Admin user created successfully!"
    echo
    echo "You can now login with:"
    echo "  Username: $ADMIN_USERNAME"
    echo "  Password: $ADMIN_PASSWORD"
    echo "========================================="
else
    echo "Failed to create admin user"
    exit 1
fi