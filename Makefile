dev-test:
	docker run -d \
      --name auth-service \
      --restart always \
      -p 3000:3000 \
      -p 50051:50051 \
      $(DOCKER_IMAGE)

	hurl --variable BASE_URL=$(BASE_URL) --jobs 1 --repeat 3 --test ./http/

grpc-test:
	@grpcurl -plaintext 127.0.0.1:50051 describe
	@echo "----"
	@grpcurl -plaintext -d '{"token": "dadsasdads"}' 127.0.0.1:50051 auth_service.AuthService/VerifyToken

grpc-proto:
	@grpcurl -plaintext -proto ./proto/auth_service.proto -d '{"token": "my-token"}' 127.0.0.1:50051 auth_service.AuthService/VerifyToken
