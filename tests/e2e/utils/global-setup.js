async function globalSetup() {
  console.log('🚀 Starting global setup for E2E tests...');

  // Wait for the server to be ready
  const maxRetries = 30;
  const baseURL = process.env.BASE_URL || 'http://localhost:3000';

  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(`${baseURL}/`);
      if (response.ok) {
        console.log('✅ Server is ready!');
        return;
      }
    } catch (error) {
      console.log(`⏳ Waiting for server... (${i + 1}/${maxRetries})`);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }

  throw new Error('❌ Server failed to start within timeout period');
}

module.exports = globalSetup;