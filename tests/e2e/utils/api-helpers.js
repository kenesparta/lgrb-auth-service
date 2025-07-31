const { expect } = require('@playwright/test');

class APIHelpers {
  constructor(request) {
    this.request = request;
    this.baseURL = process.env.BASE_URL || 'http://localhost:3000';
  }

  async signupUser(userData) {
    return await this.request.post(`${this.baseURL}/signup`, {
      data: userData
    });
  }

  async loginUser(credentials) {
    return await this.request.post(`${this.baseURL}/login`, {
      data: credentials
    });
  }

  async logoutUser(jwt) {
    return await this.request.post(`${this.baseURL}/logout`, {
      headers: {
        'Cookie': `jwt=${jwt}`
      }
    });
  }

  generateTestUser(prefix = 'test') {
    const timestamp = Date.now();
    return {
      email: `${prefix}-${timestamp}@example.com`,
      password: 'validpassword123',
      requires2FA: false
    };
  }

  async expectSuccessfulSignup(response) {
    expect(response.status()).toBe(201);
    const body = await response.json();
    expect(body.message).toBe('User created successfully!');
    return body;
  }

  async expectValidationError(response) {
    expect(response.status()).toBe(400);
    const body = await response.json();
    expect(body).toHaveProperty('error');
    return body;
  }
}

module.exports = { APIHelpers };