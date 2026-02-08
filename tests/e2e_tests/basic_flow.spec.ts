import { test, expect } from '@playwright/test';
import crypto from 'crypto';

const rootUrl = "http://localhost:3000/"
const taskDescription = "Some test task " + crypto.randomBytes(5).toString('hex');

test.describe('basic features', () => {

  test.beforeEach(async ({ page }) => {
    //Always go to the home page first
    await page.goto(rootUrl);
  });


  // Adding tasks
  test('can add a task', async ({ page }) => {
    await page.getByRole('textbox', { name: 'Priority' }).fill('B');
    await page.getByRole('textbox', { name: 'Description' }).fill(taskDescription);
    await page.getByRole('button', { name: 'Add new task' }).click();
  });

  test('can find the task again with the correct priority', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).toContain("(B)")
  });

  // Flagging as "Completed"
  test('can flag the task as completed', async ({ page }) => {
    await page.getByTestId('task-flag-completed-' + taskDescription).click();
  });

  test('does remember the task as completed', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).not.toMatch(/\([A-Z]\)/) // Priority not shown
    expect(rowText).toContain("✗") // "Completed" marker
  });

  // Flagging as "pending"
  test('can flag the task as pending', async ({ page }) => {
    await page.getByTestId('task-flag-pending-' + taskDescription).click();
  });

  test('does remember the task as pending', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).toContain("(B)") // Priority shown (and remembered)
    expect(rowText).not.toContain("✗") // "Completed" marker
  });


  // Increasing priority
  test('can increase the priority', async ({ page }) => {
    await page.getByTestId('task-increase-priority-' + taskDescription).click();
  });

  test('does remember the priority as increased', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).toContain("(A)")
  });

  // Lowering priority
  test('can lower the priority', async ({ page }) => {
    await page.getByTestId('task-lower-priority-' + taskDescription).click();
  });

  test('does remember the priority as lowered', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + taskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(taskDescription)
    expect(rowText).toContain("(B)")
  });

  // Editing description
  test('can edit the description', async ({ page }) => {
    let locator = await page.getByRole('cell', { name: taskDescription });

    await locator.click();
    await locator.pressSequentially(" some more text");
    await page.keyboard.press('Enter');
  });

  const newTaskDescription = taskDescription + " some more text";

  test('does remember new description', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + newTaskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(newTaskDescription)
  });


  // Task clean up
  test('can flag the task as completed again', async ({ page }) => {
    await page.getByTestId('task-flag-completed-' + newTaskDescription).click();
  });

  test('does remember the task as completed again', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + newTaskDescription);
    let rowText = await rowLocator.innerText();
    let rowDescriptionLocator = rowLocator.locator('input');

    expect(await rowDescriptionLocator.inputValue()).toBe(newTaskDescription)
    expect(rowText).not.toMatch(/\([A-Z]\)/) // Priority not shown
    expect(rowText).toContain("✗") // "completed" marker
  });

  test('can trigger clean up', async ({ page }) => {
    await page.getByRole('button', { name: 'Perform task cleanup' }).click();
  });

  test('does indeed remove all completed tasks', async ({ page }) => {
    let rowLocator = page.getByTestId("task-row-" + newTaskDescription);
    expect(await rowLocator.count()).toEqual(0);
  });
})