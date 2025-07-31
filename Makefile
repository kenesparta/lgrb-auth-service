dev:
	docker run -d \
      --name auth-service \
      --restart always \
      -p 3000:3000 \
      -p 50051:50051 \
      $(DOCKER_IMAGE)

	hurl --variable host=$(BASE_URL) --parallel --jobs 100 --repeat 10000 --test ./http/
