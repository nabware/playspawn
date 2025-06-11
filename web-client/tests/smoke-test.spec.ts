import { test, expect } from '@playwright/test';

test('access to web api', async ({ page, request }) => {
  const response = await request.get("http://localhost:8080/auth/sign-up");
  expect(response.status()).toBe(200);

  const body = await response.text();
  expect(body).toBe("Welcome!");
});

test('access to game api', async ({ request }) => {
  const response = await request.get("http://localhost:5001/");
  expect(response.status()).toBe(200);

  const body = await response.text();
  expect(body).toBe("Welcome!");
});
