import { test, expect } from '@playwright/test';

test.describe('Auth Service UI Tests', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(process.env.BASE_URL || 'http://localhost:3000');
    });

    test('should display the main navigation bar with logo and title', async ({ page }) => {
        const navbar = page.locator('nav.navbar');
        await expect(navbar).toBeVisible();

        const logo = page.locator('img[src="/lgr_logo.png"]');
        await expect(logo).toBeVisible();
        await expect(logo).toHaveAttribute('width', '25');
        await expect(logo).toHaveAttribute('height', '25');

        const title = page.locator('.navbar-brand');
        await expect(title).toContainText('Auth Service');
    });
});