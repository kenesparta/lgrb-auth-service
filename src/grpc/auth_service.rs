use tonic::{Request, Response, Status};

pub mod auth_service {
    tonic::include_proto!("auth_service");
}

use auth_service::{
    VerifyTokenRequest, VerifyTokenResponse,
    auth_service_server::{AuthService, AuthServiceServer},
};

use crate::utils::validate_token;

pub struct AuthServiceImpl;

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        let req = request.into_inner();
        let token = req.token;
        // TODO: Adding validation token that reads from a Database, not using a local storage.
        match validate_token(&token).await {
            Ok(_) => Ok(Response::new(VerifyTokenResponse {
                valid: true,
                message: "Token is valid".to_string(),
            })),
            Err(_) => Ok(Response::new(VerifyTokenResponse {
                valid: false,
                message: "Token is not valid".to_string(),
            })),
        }
    }
}

pub fn create_grpc_service() -> AuthServiceServer<AuthServiceImpl> {
    AuthServiceServer::new(AuthServiceImpl)
}
