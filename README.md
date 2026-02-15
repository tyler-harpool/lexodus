# Lexodus

A fullstack Rust application built with [Dioxus](https://dioxuslabs.com/) 0.7, PostgreSQL, and Axum. Features a cyberpunk-themed UI component library, server-side rendering, and interactive API documentation.

## Project Structure

```
crates/
  app/            # Frontend — routes, pages, layout, theme assets
  server/         # Backend — Dioxus server fns, REST API, database layer, OpenAPI docs
  shared-types/   # Shared data models (User, Product, DashboardStats)
  shared-ui/      # 38 cyberpunk-themed UI components wrapping dioxus-primitives
```

## Features

- **Fullstack Rust** — shared types between frontend and backend, no serialization mismatches
- **38 UI components** — cyberpunk-styled wrappers around [dioxus-primitives](https://github.com/DioxusLabs/components) (buttons, dialogs, forms, sidebar, calendar, toast notifications, and more)
- **Dark / Light theme** — toggle between cyberpunk dark and light modes via the sidebar
- **Responsive layout** — sidebar collapses to a mobile drawer on small screens
- **OpenAPI docs** — interactive Scalar API reference at `/docs` when running fullstack
- **PostgreSQL** — async database access via sqlx with compile-time checked queries
- **Offline builds** — `.sqlx/` cache allows building without a running database

## Pages

| Route        | Description                                                                     |
| ------------ | ------------------------------------------------------------------------------- |
| `/`          | **Dashboard** — statistics cards, product table with search/filter              |
| `/users`     | **Users** — CRUD user management with checkboxes, context menus, avatar badges  |
| `/products`  | **Products** — product catalog with create/edit dialogs and tab navigation      |
| `/settings`  | **Settings** — profile form, theme toggle, notifications, calendar, danger zone |

## UI Components

The `shared-ui` crate provides 38 themed components:

**Layout:** Sidebar, Navbar, Card, Separator, AspectRatio, ScrollArea, Sheet

**Forms:** Button, Input, Textarea, Checkbox, RadioGroup, Select, Slider, Switch, Toggle, ToggleGroup, Form, Label, DatePicker

**Feedback:** Dialog, AlertDialog, Toast, Tooltip, HoverCard, Popover, Progress, Skeleton, Badge

**Navigation:** Tabs, Accordion, Collapsible, Toolbar, Menubar, ContextMenu, DropdownMenu

**Data:** Avatar, Calendar

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- A container runtime for PostgreSQL — [OrbStack](https://orbstack.dev/) (recommended on macOS), [Docker Desktop](https://www.docker.com/), or [Rancher Desktop](https://rancherdesktop.io/)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/) (`cargo install dioxus-cli`)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) (`cargo install sqlx-cli --no-default-features --features postgres`)

## Quick Start

Start your container runtime, then run:

```bash
make setup
```

If you're using a non-Docker runtime like Rancher Desktop with containerd, override the compose command:

```bash
COMPOSE="nerdctl compose" make setup
```

This will:

1. Create a `.env` file from `.env.example`
2. Create a `config.toml` from `config.example.toml` (feature flags)
3. Start PostgreSQL via Docker Compose
4. Wait for the database to be ready
5. Run sqlx migrations
6. Generate the `.sqlx` offline query cache

Then start the dev server:

```bash
make web
```

The app will be available at the URL printed in the terminal (typically `http://127.0.0.1:8080`).

## Feature Flags

Optional integrations are controlled via `config.toml` (created from `config.example.toml` by `make setup`). All flags default to `false`:

| Flag        | What it enables                                        |
| ----------- | ------------------------------------------------------ |
| `oauth`     | Google & GitHub OAuth login buttons and callback routes |
| `stripe`    | Billing, checkout, subscription management              |
| `mailgun`   | Email verification, password reset emails, bounce webhooks |
| `twilio`    | SMS phone verification                                  |
| `s3`        | Avatar uploads via S3-compatible storage (MinIO / Tigris) |
| `telemetry` | OpenTelemetry traces + logs export to SigNoz            |

Enable a feature by setting it to `true` in `config.toml` and filling in the corresponding `.env` variables. The app compiles and runs with all flags off — only a database is required.

## Developer Workflow

```bash
# First time
make setup                  # Creates .env, config.toml, starts Postgres, runs migrations

# Daily development
make web                    # Start fullstack dev server (hot-reload)

# After changing SQL queries
make sqlx-prepare           # Regenerate offline cache (commit .sqlx/)

# After changing DB schema
make migrate                # Run new migrations
make sqlx-prepare           # Then regenerate cache

# Optional services (as needed)
make minio-init             # S3-compatible storage for avatar uploads
make signoz-up              # Observability dashboard at http://localhost:3301
make stripe-listen          # Forward Stripe webhooks to local server

# Before committing
make ci                     # fmt + check + clippy + test + sqlx-prepare
```

## Make Targets

### Local Dev Setup

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make setup`         | Full local dev setup (.env, config, Postgres, migrations, sqlx) |
| `make env-file`      | Create `.env` from `.env.example` (skips if exists)        |
| `make config-file`   | Create `config.toml` from `config.example.toml` (skips if exists) |

### Database

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make db-up`         | Start PostgreSQL via Docker Compose                        |
| `make db-down`       | Stop all Compose services                                  |
| `make db-wait`       | Wait for Postgres to accept connections                    |
| `make db-reset`      | Drop, recreate, and migrate the database                   |
| `make migrate`       | Run sqlx migrations                                        |
| `make sqlx-prepare`  | Regenerate `.sqlx` offline query cache                     |

### Services

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make services`      | Start all services (Postgres + MinIO + SigNoz)             |
| `make services-down` | Stop all services                                          |
| `make minio-up`      | Start MinIO (S3-compatible object storage)                 |
| `make minio-init`    | Start MinIO and create the avatars bucket                  |
| `make signoz-up`     | Start SigNoz (dashboard at `http://localhost:3301`)        |
| `make signoz-down`   | Stop SigNoz                                                |

### Stripe

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make stripe-setup`  | Full Stripe local setup (install CLI + login + listen)     |
| `make stripe-install`| Install the Stripe CLI via Homebrew                        |
| `make stripe-login`  | Authenticate the Stripe CLI with your account              |
| `make stripe-listen` | Forward Stripe webhooks to local server                    |

### Dev Servers

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make web`           | Start the Dioxus web dev server (fullstack)                |
| `make desktop`       | Start the Dioxus desktop dev server                        |
| `make mobile`        | Start the Dioxus mobile dev server (iOS simulator)         |
| `make build`         | Bundle for release                                         |

### Build & Lint

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make check`         | Cargo check (workspace, no server features)                |
| `make check-server`  | Cargo check with server features (requires DATABASE_URL)   |
| `make check-platforms` | Check all platform feature flags (web, desktop, mobile, server) |
| `make fmt`           | Format all code                                            |
| `make clippy`        | Run clippy lints                                           |
| `make test`          | Run all tests                                              |

### CI/CD & Deployment

| Command              | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `make ci`            | Run full CI checks (fmt, check, clippy, test, sqlx)        |
| `make deploy`        | Full deploy pipeline (CI + push + Fly.io)                  |
| `make git-push`      | Git add, commit, and push (prompts for message)            |
| `make fly-secrets`   | Sync `.env.production` secrets to Fly.io                   |
| `make fly-deploy`    | Deploy to Fly.io                                           |
| `make promote-user`  | Promote a user to admin (`EMAIL=user@example.com`)         |

## API Documentation

Once the dev server is running (`make web`), navigate to `/docs` for the interactive Scalar UI where you can browse and test all API endpoints.

### REST Endpoints

| Method   | Path                        | Description               |
| -------- | --------------------------- | ------------------------- |
| `POST`   | `/api/auth/register`        | Register a new user       |
| `POST`   | `/api/auth/login`           | Login with email/password |
| `POST`   | `/api/auth/logout`          | Logout (revoke tokens)    |
| `GET`    | `/api/users`                | List all users            |
| `GET`    | `/api/users/{user_id}`      | Get user by ID            |
| `POST`   | `/api/users`                | Create a user             |
| `PUT`    | `/api/users/{user_id}`      | Update a user             |
| `DELETE` | `/api/users/{user_id}`      | Delete a user             |
| `PUT`    | `/api/users/{user_id}/tier` | Update user tier (admin)  |
| `POST`   | `/api/users/me/avatar`      | Upload avatar (multipart) |
| `GET`    | `/api/products`             | List all products         |
| `POST`   | `/api/products`             | Create a product          |
| `PUT`    | `/api/products/{id}`        | Update a product          |
| `DELETE` | `/api/products/{id}`        | Delete a product          |
| `GET`    | `/api/dashboard/stats`      | Dashboard statistics      |
| `POST`   | `/api/billing/checkout`     | Create Stripe checkout session |
| `POST`   | `/api/billing/portal`       | Create Stripe customer portal session |
| `GET`    | `/api/billing/subscription` | Get current subscription status |
| `POST`   | `/api/billing/cancel`       | Cancel subscription        |
| `POST`   | `/webhooks/stripe`          | Stripe webhook receiver    |
| `GET`    | `/api/auth/verify-email`    | Verify email via token     |
| `POST`   | `/api/auth/forgot-password` | Send password reset email  |
| `POST`   | `/api/auth/reset-password`  | Reset password with token  |
| `POST`   | `/webhooks/mailgun`         | Mailgun bounce webhook     |
| `POST`   | `/api/account/send-verification` | Send SMS verification code |
| `POST`   | `/api/account/verify-phone` | Verify phone with code     |
| `GET`    | `/health`                   | Health check               |

## Theming

The app ships with 8 themes in `crates/app/assets/themes/` (one file per theme family):

| Theme             | Mode  | Description                                              |
| ----------------- | ----- | -------------------------------------------------------- |
| `cyberpunk`       | Dark  | Default — neon cyan accents on deep blue-black           |
| `light`           | Light | Clean whites with teal accents                           |
| `solar`           | Dark  | Solarized dark — warm blue-green tones                   |
| `solar-light`     | Light | Solarized light — cream backgrounds with blue accents    |
| `federal`         | Dark  | Slate navy — modern institutional application            |
| `federal-light`   | Light | Crisp whites with blue accents, high readability         |
| `chambers`        | Dark  | Warm charcoal with gold — judicial/executive feel        |
| `parchment`       | Light | Cream and forest green — document-reading optimized      |

Select a theme from the Settings page. Dual-mode themes (Cyberpunk, Solarized, Federal) also have a dark/light toggle. All 38 components automatically adapt via CSS custom properties.

### Creating a Custom Theme

Adding a new theme requires one CSS file and a small Rust change.

**Step 1 — Create a theme file** at `crates/app/assets/themes/mytheme.css`:

```css
[data-theme="solar"] {
    /* Dark/light mode flag (pick one) */
    --dark: initial;   /* set --dark for dark themes */
    --light: ;         /* leave empty for dark themes (swap for light) */

    /* Primary palette — backgrounds, surfaces, borders (dark to light) */
    --primary-color-1: #002b36;
    --primary-color-2: #073642;
    --primary-color-3: #0a3f4e;
    --primary-color-4: #0e4d5e;
    --primary-color-5: #155a6b;
    --primary-color-6: #1c6e80;
    --primary-color-7: #268399;
    --primary-color-8: #2aa198;
    --primary-color-9: #35c4ba;

    /* Secondary palette — text colors (light to dark) */
    --secondary-color-1: #fdf6e3;
    --secondary-color-2: #eee8d5;
    --secondary-color-3: #c4b99a;
    --secondary-color-4: #93a1a1;
    --secondary-color-5: #657b83;
    --secondary-color-6: #586e75;
    --secondary-color-7: #4a6068;

    /* Semantic mappings (point these at palette slots) */
    --color-background: var(--primary-color-2);
    --color-surface: var(--primary-color-3);
    --color-surface-raised: var(--primary-color-5);
    --color-surface-dialog: var(--primary-color-6);
    --color-on-surface: var(--secondary-color-1);
    --color-on-surface-muted: var(--secondary-color-4);
    --color-border: var(--primary-color-6);

    /* Accent colors */
    --color-primary: #b58900;
    --color-primary-hover: #d4a017;
    --color-on-primary: #002b36;

    --color-secondary: #6c71c4;
    --color-secondary-hover: #8a8fd6;
    --color-on-secondary: #ffffff;

    --color-danger: #dc322f;
    --color-on-danger: #ffffff;
    --color-success: #859900;
    --color-on-success: #002b36;
    --color-warning: #cb4b16;
    --color-on-warning: #ffffff;

    --focused-border-color: var(--color-primary);

    /* Shadows and glow effects */
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.4);
    --shadow-md: 0 4px 8px rgba(0, 0, 0, 0.4);
    --shadow-lg: 0 10px 20px rgba(0, 0, 0, 0.4);
    --cyber-neon-glow: 0 0 8px rgba(181, 137, 0, 0.4);
    --cyber-neon-glow-strong: 0 0 12px rgba(181, 137, 0, 0.6);
    --cyber-scanline-opacity: 0;
}
```

**Step 2 — Register it** in `crates/shared-ui/src/theme.rs`:

Add a variant to `ThemeFamily`, update `as_str()`, `display_name()`, `from_key()`, and `resolve()`.

**Step 3 — Load it** in `crates/app/src/main.rs`:

```rust
const THEME_MYTHEME: Asset = asset!("/assets/themes/mytheme.css");
// ... then add a document::Link in the App component
```

That's it. The Settings selector automatically picks up new families from `ALL_FAMILIES`. The theme persists across page reloads via a cookie and syncs across tabs.

#### Variable Reference

| Variable Group | Purpose |
| --- | --- |
| `--primary-color-1` to `--primary-color-9` | Background/surface palette (darkest to lightest) |
| `--secondary-color-1` to `--secondary-color-7` | Text palette (lightest to darkest) |
| `--color-background`, `--color-surface`, `--color-surface-raised`, `--color-surface-dialog` | Semantic surface mappings |
| `--color-on-surface`, `--color-on-surface-muted` | Text on surfaces |
| `--color-primary`, `--color-primary-hover`, `--color-on-primary` | Primary accent (buttons, links, focus rings) |
| `--color-secondary`, `--color-danger`, `--color-success`, `--color-warning` | Additional accent colors |
| `--cyber-neon-glow`, `--cyber-neon-glow-strong` | Focus/hover glow effects |
| `--cyber-scanline-opacity` | Scanline overlay on primary buttons (0 to disable) |

## Offline Builds

The `.sqlx/` directory contains cached query metadata so the project compiles without a running database. This is used by the Dockerfile (`SQLX_OFFLINE=true`) and CI. Regenerate it after changing any SQL queries:

```bash
make sqlx-prepare
```

Commit the `.sqlx/` directory after regenerating.

## Deployment

The included `Dockerfile` builds a multi-stage production image. A `fly.toml` is provided for deploying to [Fly.io](https://fly.io/).
