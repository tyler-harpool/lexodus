# ── Configuration ─────────────────────────────────────────────────────────────
ENV_FILE         := .env
ENV_EXAMPLE      := .env.example
CONFIG_FILE      := config.toml
CONFIG_EXAMPLE   := config.example.toml
SQLX_CACHE_DIR   := .sqlx
SIGNOZ_DIR       := signoz

FLY_APP    := lexodus
FLY_DB     := SET_AFTER_FLY_PG_CREATE

COMPOSE ?= docker compose

# ── Local dev setup ──────────────────────────────────────────────────────────

## Full local dev setup: .env, config.toml, Postgres, migrations, sqlx cache
setup: env-file config-file db-up db-wait migrate sqlx-prepare
	@echo "Done — local dev environment is ready."

## Copy .env.example to .env (skips if .env already exists)
env-file:
	@test -f $(ENV_FILE) || cp $(ENV_EXAMPLE) $(ENV_FILE) && echo "Created $(ENV_FILE)"

## Copy config.example.toml to config.toml (skips if config.toml already exists)
config-file:
	@test -f $(CONFIG_FILE) || cp $(CONFIG_EXAMPLE) $(CONFIG_FILE) && echo "Created $(CONFIG_FILE)"

# ── Database ─────────────────────────────────────────────────────────────────

## Start Postgres via $(COMPOSE)
db-up:
	$(COMPOSE) up -d db

## Stop Postgres
db-down:
	$(COMPOSE) down

## Wait for Postgres to accept connections
db-wait:
	@echo "Waiting for Postgres..."
	@until $(COMPOSE) exec db pg_isready -U dioxus -d lexodus > /dev/null 2>&1; do \
		sleep 1; \
	done
	@echo "Postgres is ready."

## Run sqlx migrations
migrate:
	cargo sqlx migrate run

## Generate the .sqlx offline query cache (commit this directory)
sqlx-prepare:
	cargo sqlx prepare --workspace -- --all-targets --all-features

## Reset the database (drop + recreate + migrate)
db-reset:
	cargo sqlx database drop -y
	cargo sqlx database create
	cargo sqlx migrate run

## Reset the test database (drop + recreate + migrate)
test-db-reset:
	@DB_NAME=$$(echo $$DATABASE_URL | sed 's|.*/||') && \
		TEST_DB="$${DB_NAME}_test" && \
		BASE_URL=$$(echo $$DATABASE_URL | sed 's|/[^/]*$$||') && \
		echo "Dropping $${TEST_DB}..." && \
		psql "$${BASE_URL}/postgres" -c "DROP DATABASE IF EXISTS \"$${TEST_DB}\";" && \
		echo "Test database dropped. It will be recreated on next test run."

# ── Observability (SigNoz) ───────────────────────────────────────────────────

## Clone SigNoz and start containers (dashboard at http://localhost:3301)
signoz-up:
	@if [ ! -d $(SIGNOZ_DIR) ]; then \
		git clone -b main https://github.com/SigNoz/signoz.git $(SIGNOZ_DIR); \
	fi
	cd $(SIGNOZ_DIR)/deploy/docker && $(COMPOSE) pull
	cd $(SIGNOZ_DIR)/deploy/docker && $(COMPOSE) up -d --remove-orphans

## Stop SigNoz
signoz-down:
	@if [ -d $(SIGNOZ_DIR)/deploy/docker ]; then \
		cd $(SIGNOZ_DIR)/deploy/docker && $(COMPOSE) down; \
	fi

## Start MinIO for S3-compatible object storage
minio-up:
	$(COMPOSE) up -d minio

## Create the avatars bucket in MinIO (requires mc CLI or curl)
minio-init: minio-up
	@echo "Waiting for MinIO..."
	@until curl -sf http://localhost:9000/minio/health/live > /dev/null 2>&1; do sleep 1; done
	@echo "MinIO is ready. Creating avatars bucket..."
	@curl -sf -X PUT http://minioadmin:minioadmin@localhost:9000/avatars > /dev/null 2>&1 || true
	@echo "Avatars bucket ready."

## Install Stripe CLI (macOS via Homebrew)
stripe-install:
	@command -v stripe >/dev/null 2>&1 && echo "Stripe CLI already installed." || brew install stripe/stripe-cli/stripe

## Log in to Stripe CLI
stripe-login:
	stripe login

## Forward Stripe webhooks to local server (native CLI)
stripe-listen:
	stripe listen --forward-to localhost:8080/webhooks/stripe

## Full Stripe local setup: install, login, and start listener
stripe-setup: stripe-install stripe-login stripe-listen

## Start all services (Postgres + MinIO + SigNoz)
services: db-up minio-up signoz-up

## Stop all services (Postgres, MinIO, SigNoz)
services-down: signoz-down db-down
	@echo "All services stopped."

# ── Build & check ────────────────────────────────────────────────────────────

## Cargo check (workspace, no server features)
check:
	cargo check --workspace

## Cargo check with server features (requires DATABASE_URL)
check-server:
	cargo check -p server --features server

## Cargo check all platform feature flags (web, desktop, mobile, server)
check-platforms:
	cargo check -p app --features web
	cargo check -p app --features desktop
	cargo check -p app --features mobile
	cargo check -p app --features server

## Format all code
fmt:
	cargo fmt --all

## Run clippy
clippy:
	cargo clippy --workspace -- -D warnings

# ── Test ─────────────────────────────────────────────────────────────────────

## Run all tests (shared-types parallel, server integration serialized to avoid DB pool exhaustion)
test:
	cargo test -p shared-types
	cargo test -p server --features server -- --test-threads=1

## Run Newman API tests against running local server (requires make web in another terminal)
newman:
	newman run postman/collection.json -e postman/environment.json --bail

# ── Dev server ───────────────────────────────────────────────────────────────

## Start the dioxus web dev server (fullstack with server-side telemetry)
web:
	dx serve --package app --platform web --fullstack

## Build for release
build:
	dx bundle --package app --platform web --release

## Start the dioxus mobile dev server (iOS simulator)
mobile:
	dx serve --package app --platform ios --fullstack

## Start the dioxus desktop dev server
desktop:
	dx serve --package app --platform desktop --fullstack

# ── CI/CD ────────────────────────────────────────────────────────────────────

## Run full CI pipeline: fmt, check, clippy, test, sqlx-prepare, push, deploy
deploy: ci git-push fly-secrets fly-deploy
	@echo "Deploy complete."

## Run CI checks only (no push/deploy)
ci: fmt check check-server check-platforms clippy test sqlx-prepare
	@echo "All CI checks passed."

## Git add, commit, and push to origin (prompts for commit message)
git-push:
	@if git diff --quiet && git diff --cached --quiet && [ -z "$$(git ls-files --others --exclude-standard)" ]; then \
		echo "No changes to commit — pushing existing commits."; \
	else \
		read -p "Commit message: " msg; \
		git add -A; \
		git commit -m "$$msg"; \
	fi
	git push origin $$(git branch --show-current)

## Sync .env.production secrets to Fly.io, rotating JWT_SECRET each deploy
fly-secrets:
	@test -f .env.production || (echo "Error: .env.production not found. Copy .env.example and fill in prod values." && exit 1)
	@NEW_SECRET=$$(openssl rand -base64 48) && \
		sed -i '' "s|^JWT_SECRET=.*|JWT_SECRET=$$NEW_SECRET|" .env.production && \
		echo "Rotated JWT_SECRET."
	@grep -v '^\s*#' .env.production | grep -v '^\s*$$' | flyctl secrets import --stage
	@echo "Secrets staged to Fly.io (applied on next deploy)."

## Deploy to Fly.io
fly-deploy:
	flyctl deploy --remote-only

# ── Helpers ──────────────────────────────────────────────────────────────────

## Promote a user to admin role in production (usage: make promote-user EMAIL=user@example.com)
promote-user:
	@test -n "$(EMAIL)" || (echo "Error: EMAIL is required. Usage: make promote-user EMAIL=user@example.com" && exit 1)
	@echo "Promoting $(EMAIL) to admin on $(FLY_DB)..."
	@PATH="/opt/homebrew/opt/libpq/bin:$$PATH" && export PATH && \
		echo "UPDATE users SET role = 'admin' WHERE email = '$(EMAIL)' RETURNING id, username, email, role, tier;" \
		| flyctl mpg connect $(FLY_DB)

## Show available targets
help:
	@echo "Available targets:"
	@grep -E '^## ' Makefile | sed 's/## /  /'

.PHONY: setup env-file config-file db-up db-down db-wait migrate sqlx-prepare db-reset test-db-reset \
        check check-server check-platforms fmt clippy test newman \
        web build mobile desktop help promote-user \
        signoz-up signoz-down services services-down \
        minio-up minio-init stripe-listen \
        deploy ci git-push fly-secrets fly-deploy
