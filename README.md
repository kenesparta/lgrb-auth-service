# lgrb-auth-service

Authentication service with optional email-based 2FA, HTTP (Axum) and gRPC (Tonic) interfaces, JWT cookies, and a clean
modular architecture. This README is generated from the actual codebase to ensure accuracy.

## Features

- User Signup/Login with email validation
- Optional Two-Factor Authentication (2FA) via one-time code
- JWT-based auth using secure HttpOnly cookies (access + refresh)
- RESTful HTTP API and gRPC interface for token verification
- Account deletion
- Token refresh endpoint
- Health check
- CORS configuration via env
- Docker/Compose deployment with Ubuntu Chiseled minimal image

## Architecture (high level)

- Domain: core types and business rules (Email, Password, User, TwoFACode, etc.)
- Services: pluggable stores (in-memory HashMap/HashSet) and email client abstraction
- HTTP: Axum routes wired in src/lib.rs
- gRPC: Tonic service generated from proto/auth_service.proto
- Utilities: JWT, cookie, constants, and auth helper functions

HTTP server binds to 0.0.0.0:3000 by default; gRPC server binds to 0.0.0.0:50051.

## Configuration

Environment variables (loadable via .env thanks to dotenvy):

- JWT_SECRET (required): secret used to sign JWTs
- COOKIE_DOMAIN (required): cookie Domain attribute (e.g., localhost)
- CORS_ALLOWED_ORIGINS (optional, default: http://127.0.0.1,http://localhost)
- CAPTCHA_SITE_KEY (optional): for UI integration (compose wiring)
- CAPTCHA_SECRET_KEY (optional, used by verify_captcha module)

Token and cookie parameters (from code):

- Cookie names: jwt (access), jwt-refresh (refresh)
- Access token TTL: 600 seconds
- Refresh token TTL: 3600 seconds
- Cookies are HttpOnly; SameSite=Lax; Path=/; Domain from COOKIE_DOMAIN

## Run locally

Prerequisites:

- Rust 1.86+
- Set env vars (or create a .env in repo root):
    - JWT_SECRET=my-secret
    - COOKIE_DOMAIN=localhost

Run:

- cargo run --bin auth-service
- HTTP: http://localhost:3000
- gRPC: 127.0.0.1:50051 (plaintext by default in local)

## Docker

Build and run image:

- docker build -t auth-service .
- docker run -e JWT_SECRET=... -e COOKIE_DOMAIN=localhost -p 3000:3000 -p 50051:50051 auth-service

Docker Compose (recommended):

- Ensure env vars in your shell or a .env file
- docker compose up
- Ports: 3000->3000 (HTTP), 50051->50051 (gRPC)

Image details:

- Multi-stage with cargo-chef for dependency caching
- Final image based on Ubuntu Chiseled (scratch rootfs) for minimal surface

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
5) Logout: POST /logout removes cookie and bans the token for its lifetime.

Notes:

- JWT.claims: { sub: email, exp: unix_ts, token_type: "access"|"refresh" }
- Cookies are HttpOnly; store JWTs in cookies, not localStorage.

### Curl examples

- Signup:
  curl -s -X POST http://localhost:3000/signup -H 'Content-Type: application/json' -d '{"email":"user@example.com","
  password":"password123","requires2FA":false}'

- Login (no 2FA):
  curl -i -X POST http://localhost:3000/login -H 'Content-Type: application/json' -d '{"email":"user@example.com","
  password":"password123"}'

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

- Run tests: cargo test
- Integration tests hit HTTP routes and gRPC verify (see tests/api)
- Useful Make targets:
    - make dev-test (runs container and http tests using hurl)
    - make grpc-test (grpcurl samples against running server)
    - make grpc-proto (grpcurl using local proto)

## Troubleshooting

- Panic at startup: ensure JWT_SECRET and COOKIE_DOMAIN are set (non-empty)
- CORS blocked: set CORS_ALLOWED_ORIGINS to include your frontend origin
- Cookies not set in browser: ensure COOKIE_DOMAIN matches the host you are using
- 206 on login: this indicates 2FA is enabled; proceed with /verify-2fa

## License

MIT (or the license applicable to your project)
