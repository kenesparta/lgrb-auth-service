# Let's Get Rusty bootcamp auth service

Authentication service with optional email-based 2FA, HTTP (Axum) and gRPC (Tonic) interfaces, JWT cookies, PostgreSQL
database, Redis caching, and a clean modular architecture. This README is generated from the actual codebase to ensure
accuracy.

## Features

- User Signup/Login with email validation
- Optional Two-Factor Authentication (2FA) via one-time code
- JWT-based auth using secure HttpOnly cookies (access + refresh)
- RESTFull HTTP API and gRPC interface for token verification
- PostgreSQL database with SQLx migrations for persistent user storage
- Redis integration for banned token management
- Argon2 password hashing for secure credential storage
- Multiple data store implementations (HashMap for development, PostgreSQL/Redis for production)
- Account deletion
- Token refresh endpoint
- Health check
- CORS configuration via env
- Docker/Compose deployment with Ubuntu Chiseled minimal image

## Architecture (high level)

- Domain: core types and business rules (Email, Password, User, TwoFACode, etc.)
- Services: pluggable data stores with multiple implementations:
    - HashMap stores for development/testing (in-memory)
    - PostgreSQL store for persistent user data (production)
    - Redis store for banned token management (production)
    - AWS SES email client for production email delivery
    - Mock email client for development and testing
- Database: PostgreSQL with SQLx migrations for schema management
- HTTP: Axum routes wired in src/lib.rs with comprehensive error handling
- gRPC: Tonic service generated from proto/auth_service.proto
- Utilities: JWT, cookie management, constants, and auth helper functions
- App State: centralized application state management

HTTP server binds to `0.0.0.0:3000` by default; gRPC server binds to `0.0.0.0:50051`.

## Configuration

The application supports two configuration methods:

### Environment Variables

Environment variables (loadable via .env thanks to dotenvy) - all use AUTH_LGRB_ prefix:

**Required:**

- AUTH_LGRB_JWT_SECRET: secret used to sign JWTs
- AUTH_LGRB_COOKIE_DOMAIN: cookie Domain attribute (e.g., localhost)
- AUTH_LGRB_DATABASE_URL: PostgreSQL connection string (e.g., postgresql://user:password@localhost/dbname)

**Optional:**

- AUTH_LGRB_CORS_ALLOWED_ORIGINS (default: http://127.0.0.1,http://localhost): comma-separated list of allowed origins
- AUTH_LGRB_REDIS_HOST_NAME (default: 127.0.0.1): Redis server hostname for banned token storage
- AUTH_LGRB_CAPTCHA_SITE_KEY: for UI integration (compose wiring)
- AUTH_LGRB_CAPTCHA_SECRET_KEY: used by verify_captcha module
- AUTH_LGRB_TOKEN_TTL_SECONDS (default: 600): access token lifetime in seconds
- AUTH_LGRB_REFRESH_TOKEN_TTL_SECONDS (default: 3600): refresh token lifetime in seconds
- AUTH_LGRB_POSTGRES_PASSWORD: PostgreSQL password for containerized deployments

### YAML Configuration

Alternative configuration via config.yaml file with cleaner syntax:

```yaml
jwt_secret: "your-secret"
cookie_domain: "localhost"
database_url: "postgresql://user:password@localhost/dbname"
redis_host_name: "127.0.0.1"
cors_allowed_origins: "http://localhost,http://127.0.0.1"
captcha_site_key: "your-key"
captcha_secret_key: "your-secret"
token_ttl_seconds: 600
refresh_token_ttl_seconds: 3600
```

Token and cookie parameters:

- Cookie names: jwt (access), jwt-refresh (refresh)
- Access token TTL: configurable via AUTH_LGRB_TOKEN_TTL_SECONDS (default: 600 seconds)
- Refresh token TTL: configurable via AUTH_LGRB_REFRESH_TOKEN_TTL_SECONDS (default: 3600 seconds)
- Cookies are HttpOnly; SameSite=Lax; Path=/; Domain from AUTH_LGRB_COOKIE_DOMAIN

## Run locally

Prerequisites:

- Rust 1.86+ (edition 2024)
- PostgreSQL 12+ (for persistent storage)
- Redis 6+ (optional, for production-like banned token storage)
- Set env vars (or create a .env in repo root):
    - AUTH_LGRB_JWT_SECRET=my-secret
    - AUTH_LGRB_COOKIE_DOMAIN=localhost
    - AUTH_LGRB_DATABASE_URL=postgresql://user:password@localhost/auth_db

Database setup:

1. Create PostgreSQL database: `createdb auth_db`
2. Run migrations: `cargo install sqlx-cli && sqlx migrate run`

Run:

- cargo run --bin auth-service
- HTTP: http://localhost:3000
- gRPC: 127.0.0.1:50051 (plaintext by default in local)

For development with in-memory stores (no database required), the application will fall back to HashMap implementations
when database connections fail.

## Docker

Build and run image:

- `docker build -t auth-service .`
-

`docker run -e AUTH_LGRB_JWT_SECRET=... -e AUTH_LGRB_COOKIE_DOMAIN=localhost -e AUTH_LGRB_DATABASE_URL=postgresql://... -p 3000:3000 -p 50051:50051 auth-service`

Docker Compose (recommended):

- Ensure env vars in your shell or a `.env` file (includes AUTH_LGRB_DATABASE_URL for PostgreSQL)
- docker compose up
- Ports: 3000->3000 (HTTP), 50051->50051 (gRPC)
- Compose includes PostgreSQL 17.6 and Redis 8.2 Alpine services
- All environment variables use AUTH_LGRB_ prefix for consistency

Image details:

- Multi-stage with cargo-chef for dependency caching
- Final image based on Ubuntu Chiseled (scratch rootfs) for minimal surface

## Email Service Configuration

The application supports two email service implementations:

### AWS SES (Production)

- **Default in production**: AWS Simple Email Service for reliable email delivery
- **Region**: Currently configured for `us-east-1` (hardcoded in main.rs)
- **From Email**: Currently set to `auth@rustybootcamp.xyz` (hardcoded in main.rs)
- **AWS Credentials**: Uses default AWS credential chain (IAM roles, environment variables, etc.)
- **Permissions Required**: `ses:SendEmail` for the configured from address

### Mock Email Client (Development/Testing)

- **Default in tests**: In-memory mock that logs email attempts without sending
- **Automatic fallback**: Used when AWS SES client creation fails
- **No configuration required**: Works out of the box for development

## Database Migrations

The project uses SQLx for database migrations located in the `migrations/` directory:

- `20250816151919_create_users_table.up.sql`: Creates the users table with email, password_hash, and requires_2fa fields
- Migrations are automatically applied on application startup in production
- For local development: `sqlx migrate run`
- To revert: `sqlx migrate revert`

Database schema:

```sql
CREATE TABLE users
(
    email         TEXT PRIMARY KEY,
    password_hash TEXT    NOT NULL,
    requires_2fa  BOOLEAN NOT NULL DEFAULT FALSE
);
```

## REST API

The OpenAPI schema lives at api_schema.yml (importable in Swagger Editor). Key endpoints wired in src/lib.rs:

- GET /
    - Serves static assets from ./assets
- GET /health-check
    - 200 OK
- POST /signup
    - Body: { "email": string, "password": string (>=8 chars), "requires2FA": boolean }
    - 201 Created on success
    - 400 if validation fails; 409 if user exists
- POST /login
    - Body: { "email": string, "password": string }
    - 200 OK + Set-Cookie: jwt, jwt-refresh when 2FA is not required
    - 206 Partial Content when 2FA is required with JSON: { message, loginAttemptId }
    - 400/401 on failures
- POST /verify-2fa
    - Body: { "email": string, "loginAttemptId": string, "2FACode": string(6 digits) }
    - 200 OK + Set-Cookie: jwt, jwt-refresh on success
    - 400 if malformed inputs; 401 if incorrect
- POST /refresh-token
    - Reads jwt-refresh cookie, must be a valid refresh token
    - 200 OK + sets fresh jwt and jwt-refresh cookies
    - JSON response: { message, access_token, refresh_token }
    - 400/401 on missing/invalid token
- POST /logout
    - Requires jwt cookie; validates it, bans token, then clears cookie
    - 200 OK on success; 400 if missing token; 401 if invalid
- DELETE /delete-account
    - Body: { "email": string }
    - 204 No Content on success

### Auth and 2FA flow

1) Signup: POST /signup with requires2FA true/false.
2) Login: POST /login
    - If requires2FA=false: 200 with cookies.
    - If requires2FA=true: 206 with loginAttemptId; a code is emailed (MockEmailClient during dev/tests).
3) Verify: POST /verify-2fa with email, loginAttemptId, and 2FACode. On success, cookies are set.
4) Refresh: POST /refresh-token when access token expires to rotate cookies.
5) Logout: POST /logout removes the cookie and bans the token for its lifetime.

Notes:

- JWT.claims: { sub: email, exp: unix_ts, token_type: "access"|"refresh" }
- Cookies are HttpOnly; store JWTs in cookies, not localStorage.

### Curl examples

- Signup:
  ```
  curl -s -X POST http://localhost:3000/signup -H 'Content-Type: application/json' -d '{"email":"user@example.com","password":"password123","requires2FA":false}'
  ```

- Login (no 2FA):
  ```
  curl -i -X POST http://localhost:3000/login -H 'Content-Type: application/json' -d '{"email":"user@example.com","password":"password123"}'
  ```

- Refresh tokens:
  curl -i -X POST http://localhost:3000/refresh-token --cookie "jwt-refresh=..."

- Logout:
  curl -i -X POST http://localhost:3000/logout --cookie "jwt=..."

## gRPC API

- Proto: proto/auth_service.proto
- Service: auth_service.AuthService
- Method: VerifyToken(VerifyTokenRequest) -> VerifyTokenResponse
- Example with grpcurl:
    - grpcurl -plaintext 127.0.0.1:50051 describe
    - grpcurl -plaintext -d '{"token":"your-token"}' 127.0.0.1:50051 auth_service.AuthService/VerifyToken

Server reflection is enabled in main (tonic_reflection), so you can use grpcurl without proto.

## CORS

- Configure with CORS_ALLOWED_ORIGINS comma-separated list (defaults to http://127.0.0.1,http://localhost). Credentials
  are enabled.

## CAPTCHA (optional)

A simple Google reCAPTCHA v3 verification helper exists (src/routes/verify_captcha.rs). To use it set:

- CAPTCHA_SITE_KEY, CAPTCHA_SECRET_KEY

## Development & Testing

- Run tests: `cargo test`
- Integration tests hit HTTP routes and gRPC verify (see tests/api)
- Tests use HashMap data stores (in-memory) for fast execution without database dependencies
- For database-dependent testing, ensure PostgreSQL is running and DATABASE_URL is set
- Useful Make targets:
    - `make dev-test` (runs container and http tests using hurl)
    - `make grpc-test` (grpcurl samples against running server)
    - `make grpc-proto` (grpcurl using local proto)

Test environment setup:

- Tests automatically use in-memory HashMap stores
- No database setup required for unit/integration tests
- For full stack testing with database: set AUTH_LGRB_DATABASE_URL and run migrations

## Troubleshooting

- Panic at startup: ensure AUTH_LGRB_JWT_SECRET, AUTH_LGRB_COOKIE_DOMAIN, and AUTH_LGRB_DATABASE_URL are set (non-empty)
- Database connection failed: verify PostgreSQL is running and AUTH_LGRB_DATABASE_URL is correct
- Migration errors: ensure database exists and run `sqlx migrate run`
- Redis connection issues: check AUTH_LGRB_REDIS_HOST_NAME or use HashMap stores for development
- CORS blocked: set AUTH_LGRB_CORS_ALLOWED_ORIGINS to include your frontend origin
- Cookies aren't set in browser: ensure AUTH_LGRB_COOKIE_DOMAIN matches the host you are using
- 206 on login: this indicates 2FA is enabled; proceed with /verify-2fa
- Application falls back to HashMap stores: this occurs when database connections fail (acceptable for development)
- AWS SES email errors: verify AWS credentials and SES permissions for the configured from address
- Email service fallback: application uses mock email client when AWS SES initialization fails

## License

MIT
