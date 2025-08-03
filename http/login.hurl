# ###
POST {{BASE_URL}}/signup
Content-Type: application/json

{
  "email": "my+email@example.com",
  "password": "password123",
  "requires2FA": false
}

HTTP 201
[Asserts]
jsonpath "$.message" == "User created successfully!"

# ###
POST {{BASE_URL}}/login
Content-Type: application/json

{
  "email": "my+email@example.com",
  "password": "12345678ABC"
}

HTTP 401
[Asserts]
jsonpath "$.error_message" == "Incorrect credentials"