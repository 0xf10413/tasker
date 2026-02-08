import { test, expect } from '@playwright/test';
import crypto from 'crypto';

const rootUrl = "http://localhost:3000/"
const preset = "Preset " + crypto.randomBytes(2).toString('hex');
const taskDescription = "Some project task " + crypto.randomBytes(5).toString('hex');


test.describe('preset features', () => {

  test.beforeEach(async ({ page }) => {
    //Always go to the home page first
    await page.goto(rootUrl);
  });


  test('can add a new preset', async ({ page }) => {
    await page.getByRole('textbox', { name: 'New preset name' }).click();
    await page.getByRole('textbox', { name: 'New preset name' }).fill(preset);
    await page.getByRole('button', { name: 'Add new preset' }).click(); // Navigation
  })

  test('can find the preset again in the home page', async ({ page }) => {
    expect(page.getByRole('button', { name: preset })).toBeDefined();
  })

  test('can add a task in the preset', async ({ page }) => {
    await page.getByRole('button', { name: preset }).click();

    await page.getByRole('textbox', { name: 'Priority' }).click();
    await page.getByRole('textbox', { name: 'Priority' }).fill('A');
    await page.getByRole('textbox', { name: 'Description' }).click();
    await page.getByRole('textbox', { name: 'Description' }).fill(taskDescription);
    await page.getByRole('button', { name: 'Add preset task' }).click();
  });

  test('can find the task again in the preset', async ({ page }) => {
    await page.getByRole('button', { name: preset }).click();

    let rowLocator = page.getByTestId("preset-task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    expect(rowText).toContain(taskDescription);
  });

  test('can inject the preset', async ({ page }) => {
    await page.getByRole('button', { name: preset }).click();
    await page.getByRole('button', { name: 'Inject preset' }).click();
  });

  test('can find the injected tasks in the main page', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
  });

})