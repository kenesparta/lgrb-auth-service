const { test, expect } = require('@playwright/test');
const { APIHelpers } = require('../../utils/api-helpers');
const testData = require('../../fixtures/test-data.json');

test.describe('Signup API', () => {
  let apiHelpers;

  test.beforeEach(async ({ request }) => {
    apiHelpers = new APIHelpers(request);
  });

  test('should successfully create a new user', async () => {
    const userData = apiHelpers.generateTestUser('signup');
    const response = await apiHelpers.signupUser(userData);
    await apiHelpers.expectSuccessfulSignup(response);
  });

  test('should create user with 2FA enabled', async () => {
    const userData = apiHelpers.generateTestUser('signup2fa');
    userData.requires2FA = true;

    const response = await apiHelpers.signupUser(userData);
    await apiHelpers.expectSuccessfulSignup(response);
  });

  test.describe('Validation Tests', () => {
    test('should reject empty email', async () => {
      const userData = {
        email: '',
        password: 'validpassword123',
        requires2FA: false
      };

      const response = await apiHelpers.signupUser(userData);
      await apiHelpers.expectValidationError(response);
    });

    test('should reject short password', async () => {
      const userData = {
        email: 'test@example.com',
        password: 'short',
        requires2FA: false
      };

      const response = await apiHelpers.signupUser(userData);
      await apiHelpers.expectValidationError(response);
    });

    testData.invalidEmails.forEach(email => {
      test(`should reject invalid email: "${email}"`, async () => {
        const userData = {
          email: email,
          password: 'validpassword123',
          requires2FA: false
        };

        const response = await apiHelpers.signupUser(userData);
        await apiHelpers.expectValidationError(response);
      });
    });
  });

  test('should handle duplicate user registration', async () => {
    const userData = apiHelpers.generateTestUser('duplicate');

    // The first signup should succeed
    const firstResponse = await apiHelpers.signupUser(userData);
    await apiHelpers.expectSuccessfulSignup(firstResponse);

    // The second signup should fail
    const secondResponse = await apiHelpers.signupUser(userData);
    expect(secondResponse.status()).toBe(409);
  });
});