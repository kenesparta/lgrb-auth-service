I'll help you create a comprehensive README for this project. Let me first explore the project structure to understand
the complete scope.# Auth Service
A robust authentication service built in Rust using Axum for HTTP REST API and Tonic for gRPC functionality. This
service provides comprehensive user management with features like user registration, login, 2FA verification, and
account management.

## Features

- **User Authentication**: Secure user registration and login with email validation
- **Two-Factor Authentication (2FA)**: Optional 2FA support for enhanced security
- **RESTful API**: HTTP endpoints for all authentication operations
- **gRPC Support**: High-performance gRPC interface alongside REST API
- **Account Management**: User account deletion and management
- **Token Verification**: JWT token validation and verification
- **Health Monitoring**: Health check endpoints for service monitoring

## Architecture

The service follows a clean architecture pattern with:

- **Domain Layer**: Core business logic and entities (User, Email, Password)
- **Service Layer**: Application services and business operations
- **Data Store**: Pluggable storage layer with HashMap implementation
- **Routes**: HTTP REST API endpoints
- **gRPC**: Protocol buffer-based gRPC services

## API Endpoints

- `GET /` - Serves static assets
- `GET /health-check` - Health status endpoint
- `POST /signup` - User registration
- `POST /login` - User authentication
- `POST /logout` - User logout
- `POST /verify-2fa` - Two-factor authentication verification
- `POST /verify-token` - JWT token verification
- `DELETE /delete-account` - Account deletion

## Technology Stack

- **Framework**: Axum for HTTP server
- **gRPC**: Tonic for high-performance RPC
- **Async Runtime**: Tokio
- **Serialization**: Serde for JSON, Prost for Protocol Buffers
- **Validation**: Validator for email validation
- **Error Handling**: thiserror for structured error types
- **Testing**: Mockall for mocking, quickcheck for property-based testing
- **Containerization**: Multi-stage Docker build with Ubuntu Chiseled images

## Getting Started

### Prerequisites

- Rust 1.86 or later
- Docker (optional, for containerized deployment)

### Running Locally

``` bash
cargo run --bin auth-service
```

### Running with Docker

``` bash
docker build -t auth-service .
docker run -p 8080:8080 auth-service
```

### Running with Docker Compose

``` bash
docker compose up
```

## Development

The project includes comprehensive testing with unit tests and integration tests. Run tests with:

``` bash
cargo test
```

For development, the service supports hot reloading and includes extensive error handling and logging.

## Security Features

- Email format validation using the validator crate
- Secure password handling
- Two-factor authentication support
- Input validation and sanitization
- Structured error responses without sensitive information leakage

## Docker Deployment

The service uses a multi-stage Docker build process:

1. **Chef Stage**: Caches Rust dependencies for faster builds
2. **Builder Stage**: Compiles the application
3. **Chiseled Ubuntu**: Creates a minimal, secure runtime image

The final image is based on Ubuntu Chiseled for security and minimal attack surface, containing only the necessary
runtime dependencies.
