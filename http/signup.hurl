# ###
POST http://localhost:3000/signup
Content-Type: application/json

{
  "email": "{{newUuid}}@example.com",
  "password": "password123",
  "requires2FA": false
}

HTTP 201
[Asserts]
jsonpath "$.message" == "User created successfully!"

# ###
# Test signup with invalid email (400)
POST {{host}}/signup
Content-Type: application/json

{
  "email": "invalid-email",
  "password": "password123",
  "requires2FA": false
}

HTTP 400
[Asserts]
jsonpath "$.error_message" exists

# ###
# Test signup with existing email (409)
POST {{host}}/signup
Content-Type: application/json

{
  "email": "user001example.com",
  "password": "12",
  "requires2FA": false
}

HTTP 400
[Asserts]
jsonpath "$.error_message" exists
