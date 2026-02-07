import { test, expect } from '@playwright/test';
import crypto from 'crypto';

const rootUrl = "http://localhost:3000/"
const projectTaskDescription = "Some project task " + crypto.randomBytes(5).toString('hex');
const project = "proj" + crypto.randomBytes(2).toString('hex');
const taskDescription = "Some project task " + crypto.randomBytes(5).toString('hex');


test.describe('basic features', () => {

  test.beforeEach(async ({ page }) => {
    //Always go to the home page first
    await page.goto(rootUrl);
  });


  // Adding tasks with project
  test('can add a task with a project', async ({ page }) => {
    await page.getByRole('textbox', { name: 'Priority' }).fill('B');
    await page.getByRole('textbox', { name: 'Description' }).fill(projectTaskDescription);
    await page.getByPlaceholder('Project').fill(project);
    await page.getByRole('button', { name: 'Add' }).click();
  });

  test('can find the task again with the right project', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + projectTaskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(projectTaskDescription)
    expect(rowText).toContain("(B)")
    expect(rowText).toContain(project)
  });


  // Project filtering
  test('can add a task without any project', async ({ page }) => {
    await page.getByRole('textbox', { name: 'Priority' }).fill('A');
    await page.getByRole('textbox', { name: 'Description' }).fill(taskDescription);
    await page.getByRole('button', { name: 'Add' }).click();
  });

  test('can find the task again without any project', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).toContain("(A)")
  });

  test('can filter tasks only on relevant project', async ({ page }) => {
    await page.getByRole('button', { name: project }).click(); // Should trigger navigation

    let rowTaskLocator = page.getByTestId("task-row-" + taskDescription);
    let rowProjectTaskLocator = page.getByTestId("task-row-" + projectTaskDescription);

    expect(await rowTaskLocator.count()).toEqual(0);
    expect(await rowProjectTaskLocator.count()).toEqual(1);
  });

})