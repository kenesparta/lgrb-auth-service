// Import commands for both API and browser testing
import './api-commands'
import './browser-commands'

// Global configurations
Cypress.on('uncaught:exception', (err, runnable) => {
  // Prevent Cypress from failing on uncaught exceptions
  return false
})

// Add custom commands or global hooks here
beforeEach(() => {
  // Common setup for all tests
})