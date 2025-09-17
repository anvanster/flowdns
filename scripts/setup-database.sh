#!/bin/bash

# Database setup script for FlowDNS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default database configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-flowdns}"
DB_USER="${DB_USER:-flowdns}"
DB_PASSWORD="${DB_PASSWORD:-Admin@12}"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

# Function to URL encode special characters
urlencode() {
    local string="${1}"
    local strlen=${#string}
    local encoded=""
    local pos c o

    for (( pos=0 ; pos<strlen ; pos++ )); do
        c=${string:$pos:1}
        case "$c" in
            [-_.~a-zA-Z0-9] ) o="${c}" ;;
            * ) printf -v o '%%%02x' "'$c" ;;
        esac
        encoded+="${o}"
    done
    echo "${encoded}"
}

# Function to check if PostgreSQL is installed
check_postgres() {
    if ! command -v psql &> /dev/null; then
        print_error "PostgreSQL client is not installed"
        echo "Please install PostgreSQL:"
        echo "  Ubuntu/Debian: sudo apt-get install postgresql postgresql-client"
        echo "  RHEL/CentOS:   sudo yum install postgresql postgresql-server"
        echo "  macOS:         brew install postgresql"
        exit 1
    fi
    print_status "PostgreSQL client found"
}

# Function to check if PostgreSQL server is running
check_postgres_running() {
    if pg_isready -h "$DB_HOST" -p "$DB_PORT" > /dev/null 2>&1; then
        print_status "PostgreSQL server is running on $DB_HOST:$DB_PORT"
        return 0
    else
        print_error "PostgreSQL server is not running on $DB_HOST:$DB_PORT"
        echo "Please start PostgreSQL service:"
        echo "  sudo systemctl start postgresql"
        echo "  or"
        echo "  sudo service postgresql start"
        exit 1
    fi
}

# Function to create database and user
setup_database() {
    print_info "Setting up database '$DB_NAME' with user '$DB_USER'"

    # Try to connect as postgres user (superuser)
    print_info "Connecting to PostgreSQL as superuser..."

    # Create user if it doesn't exist
    sudo -u postgres psql <<EOF 2>/dev/null || true
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = '$DB_USER') THEN
        CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
    ELSE
        ALTER USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
    END IF;
END
\$\$;
EOF

    if [ $? -eq 0 ]; then
        print_status "User '$DB_USER' created/updated successfully"
    else
        print_warning "Could not create/update user (may already exist)"
    fi

    # Create database if it doesn't exist
    sudo -u postgres psql <<EOF 2>/dev/null || true
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\\gexec
EOF

    if [ $? -eq 0 ]; then
        print_status "Database '$DB_NAME' created successfully"
    else
        print_info "Database may already exist"
    fi

    # Grant all privileges
    sudo -u postgres psql <<EOF 2>/dev/null || true
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
ALTER DATABASE $DB_NAME OWNER TO $DB_USER;
EOF

    print_status "Privileges granted to user '$DB_USER'"
}

# Function to run migrations
run_migrations() {
    print_info "Running database migrations..."

    # Check if sqlx-cli is installed
    if ! command -v sqlx &> /dev/null; then
        print_warning "sqlx-cli not installed. Installing..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    cd "$PROJECT_ROOT"

    # URL encode the password
    ENCODED_PASSWORD=$(urlencode "$DB_PASSWORD")
    DATABASE_URL="postgresql://${DB_USER}:${ENCODED_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

    # Export DATABASE_URL for sqlx
    export DATABASE_URL

    # Run migrations
    if [ -d "migrations" ] && [ "$(ls -A migrations)" ]; then
        sqlx migrate run
        if [ $? -eq 0 ]; then
            print_status "Migrations completed successfully"
        else
            print_error "Migration failed"
            exit 1
        fi
    else
        print_warning "No migrations found in $PROJECT_ROOT/migrations"
    fi
}

# Function to test connection
test_connection() {
    print_info "Testing database connection..."

    # URL encode the password
    ENCODED_PASSWORD=$(urlencode "$DB_PASSWORD")

    # Test connection with encoded password
    PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT version();" > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        print_status "Database connection successful!"

        # Update .env file with properly encoded URL
        print_info "Updating .env file with correct database URL..."
        DATABASE_URL="postgresql://${DB_USER}:${ENCODED_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

        # Create or update .env file
        cat > "$PROJECT_ROOT/.env" <<EOF
# Database configuration
DATABASE_URL=$DATABASE_URL

# API configuration
API_PORT=8080
API_HOST=0.0.0.0

# JWT Secret (change this in production!)
JWT_SECRET=your-secret-key-change-this-in-production

# Frontend URL
FRONTEND_URL=http://localhost:3000
EOF

        print_status ".env file updated with correct database URL"
        echo
        print_info "Database URL (for reference):"
        echo "  $DATABASE_URL"
    else
        print_error "Failed to connect to database"
        echo "Please check your PostgreSQL configuration and try again"
        exit 1
    fi
}

# Main execution
main() {
    echo "========================================="
    echo "FlowDNS Database Setup"
    echo "========================================="
    echo

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --host)
                DB_HOST="$2"
                shift 2
                ;;
            --port)
                DB_PORT="$2"
                shift 2
                ;;
            --name)
                DB_NAME="$2"
                shift 2
                ;;
            --user)
                DB_USER="$2"
                shift 2
                ;;
            --password)
                DB_PASSWORD="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [options]"
                echo
                echo "Options:"
                echo "  --host HOST         Database host (default: localhost)"
                echo "  --port PORT         Database port (default: 5432)"
                echo "  --name NAME         Database name (default: flowdns)"
                echo "  --user USER         Database user (default: flowdns)"
                echo "  --password PASSWORD Database password (default: Admin@12)"
                echo "  --help             Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Display configuration
    print_info "Database Configuration:"
    echo "  Host:     $DB_HOST"
    echo "  Port:     $DB_PORT"
    echo "  Database: $DB_NAME"
    echo "  User:     $DB_USER"
    echo "  Password: [hidden]"
    echo

    # Run setup steps
    check_postgres
    check_postgres_running
    setup_database
    test_connection
    run_migrations

    echo
    echo "========================================="
    echo -e "${GREEN}Database setup completed successfully!${NC}"
    echo "========================================="
    echo
    echo "You can now start the FlowDNS backend with:"
    echo "  ./scripts/start-backend.sh"
    echo
    echo "Or start all services with:"
    echo "  ./scripts/start-all.sh"
}

# Run main function
main "$@"