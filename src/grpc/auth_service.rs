use tonic::{Request, Response, Status};

pub mod auth_service {
    tonic::include_proto!("auth_service");
}

use auth_service::{
    auth_service_server::{AuthService, AuthServiceServer},
    VerifyTokenRequest, VerifyTokenResponse,
};

pub struct AuthServiceImpl;

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        let req = request.into_inner();

        let response = VerifyTokenResponse {
            valid: !req.token.is_empty(),
            message: if !req.token.is_empty() {
                "Token is valid".to_string()
            } else {
                "Token is invalid".to_string()
            },
        };

        Ok(Response::new(response))
    }
}

pub fn create_grpc_service() -> AuthServiceServer<AuthServiceImpl> {
    AuthServiceServer::new(AuthServiceImpl)
}
