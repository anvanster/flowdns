.PHONY: help install build run test clean dev docker-build docker-up docker-down

# Variables
CARGO = cargo
DOCKER = docker
DOCKER_COMPOSE = docker-compose

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[1;33m
NC = \033[0m # No Color

help: ## Show this help message
	@echo "FlowDNS Makefile Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick Start:"
	@echo "  $$ make install    # Install dependencies"
	@echo "  $$ make build      # Build the project"
	@echo "  $$ make run        # Run the server"

install: ## Install all dependencies
	@echo "$(YELLOW)Installing dependencies...$(NC)"
	@./install.sh

install-dev: ## Install development dependencies
	@echo "$(YELLOW)Setting up development environment...$(NC)"
	@./scripts/dev-setup.sh

build: ## Build the project in release mode
	@echo "$(YELLOW)Building FlowDNS...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN)Build complete!$(NC)"

build-debug: ## Build the project in debug mode
	@echo "$(YELLOW)Building FlowDNS (debug)...$(NC)"
	@$(CARGO) build
	@echo "$(GREEN)Debug build complete!$(NC)"

run: ## Run the server with default config
	@echo "$(YELLOW)Starting FlowDNS...$(NC)"
	@$(CARGO) run --release -- --config config/server.toml

run-debug: ## Run the server in debug mode
	@echo "$(YELLOW)Starting FlowDNS (debug)...$(NC)"
	@RUST_LOG=debug $(CARGO) run -- --config config/dev.toml

dev: ## Run in development mode with auto-reload
	@echo "$(YELLOW)Starting development server...$(NC)"
	@cargo watch -x 'run -- --config config/dev.toml'

test: ## Run all tests
	@echo "$(YELLOW)Running tests...$(NC)"
	@$(CARGO) test
	@echo "$(GREEN)All tests passed!$(NC)"

test-watch: ## Run tests with auto-reload
	@cargo watch -x test

clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@$(CARGO) clean
	@rm -rf target/
	@echo "$(GREEN)Clean complete!$(NC)"

migrate: ## Run database migrations
	@echo "$(YELLOW)Running migrations...$(NC)"
	@./target/release/flowdns --migrate || ./target/debug/flowdns --migrate
	@echo "$(GREEN)Migrations complete!$(NC)"

db-reset: ## Reset the development database
	@echo "$(YELLOW)Resetting development database...$(NC)"
	@sudo -u postgres psql -c "DROP DATABASE IF EXISTS flowdns_dev;" 2>/dev/null || true
	@sudo -u postgres psql -c "CREATE DATABASE flowdns_dev;" 2>/dev/null || true
	@echo "$(GREEN)Database reset complete!$(NC)"

clippy: ## Run clippy linter
	@echo "$(YELLOW)Running clippy...$(NC)"
	@$(CARGO) clippy -- -D warnings

fmt: ## Format code
	@echo "$(YELLOW)Formatting code...$(NC)"
	@$(CARGO) fmt
	@echo "$(GREEN)Formatting complete!$(NC)"

fmt-check: ## Check code formatting
	@echo "$(YELLOW)Checking formatting...$(NC)"
	@$(CARGO) fmt -- --check

check: ## Run all checks (test, clippy, fmt)
	@echo "$(YELLOW)Running all checks...$(NC)"
	@$(CARGO) test --quiet
	@$(CARGO) clippy -- -D warnings
	@$(CARGO) fmt -- --check
	@echo "$(GREEN)All checks passed!$(NC)"

docs: ## Generate and open documentation
	@echo "$(YELLOW)Generating documentation...$(NC)"
	@$(CARGO) doc --open

docker-build: ## Build Docker image
	@echo "$(YELLOW)Building Docker image...$(NC)"
	@$(DOCKER_COMPOSE) -f docker/docker-compose.yml build
	@echo "$(GREEN)Docker build complete!$(NC)"

docker-up: ## Start Docker containers
	@echo "$(YELLOW)Starting Docker containers...$(NC)"
	@$(DOCKER_COMPOSE) -f docker/docker-compose.yml up -d
	@echo "$(GREEN)Containers started!$(NC)"
	@echo "Services available at:"
	@echo "  - DHCP: Port 67/68"
	@echo "  - DNS: Port 53"
	@echo "  - API: http://localhost:8080"
	@echo "  - Adminer: http://localhost:8081"

docker-down: ## Stop Docker containers
	@echo "$(YELLOW)Stopping Docker containers...$(NC)"
	@$(DOCKER_COMPOSE) -f docker/docker-compose.yml down
	@echo "$(GREEN)Containers stopped!$(NC)"

docker-logs: ## View Docker logs
	@$(DOCKER_COMPOSE) -f docker/docker-compose.yml logs -f

docker-clean: ## Remove Docker containers and volumes
	@echo "$(YELLOW)Cleaning Docker resources...$(NC)"
	@$(DOCKER_COMPOSE) -f docker/docker-compose.yml down -v
	@echo "$(GREEN)Docker resources cleaned!$(NC)"

release: ## Build release binaries
	@echo "$(YELLOW)Building release binaries...$(NC)"
	@$(CARGO) build --release --target x86_64-unknown-linux-gnu
	@echo "$(GREEN)Release build complete!$(NC)"
	@echo "Binary location: target/x86_64-unknown-linux-gnu/release/flowdns"

install-hooks: ## Install git hooks
	@echo "$(YELLOW)Installing git hooks...$(NC)"
	@cp scripts/pre-commit .git/hooks/pre-commit 2>/dev/null || true
	@chmod +x .git/hooks/pre-commit 2>/dev/null || true
	@echo "$(GREEN)Git hooks installed!$(NC)"

version: ## Show version information
	@echo "FlowDNS Version Information:"
	@grep "^version" Cargo.toml | head -1
	@echo "Rust version: $$(rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "Cargo version: $$(cargo --version 2>/dev/null || echo 'Not installed')"

quickstart: ## Run quickstart setup
	@./quickstart.sh

.DEFAULT_GOAL := help