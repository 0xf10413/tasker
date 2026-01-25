import { test, expect } from '@playwright/test';
import crypto from 'crypto';

const rootUrl = "http://localhost:3000/"
const taskDescription = "Some project task " + crypto.randomBytes(5).toString('hex');
const taskProject = "project" + crypto.randomBytes(5).toString('hex');

test.describe('basic features', () => {

  test.beforeEach(async ({ page }) => {
    //Always go to the home page first
    await page.goto(rootUrl);
  });


  // Adding tasks
  test('can add a task with a project', async ({ page }) => {
    await page.getByRole('textbox', { name: 'Priority' }).fill('B');
    await page.getByRole('textbox', { name: 'Description' }).fill(taskDescription);
    await page.getByRole('textbox', { name: 'Project' }).fill(taskProject);
    await page.getByRole('button', { name: 'Add' }).click();
  });

  test('can find the task again with the right project', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).toContain("(B)")
    expect(rowText).toContain(taskProject)
  });

})