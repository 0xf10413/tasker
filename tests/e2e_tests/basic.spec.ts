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
    await page.getByRole('textbox', { name: 'Priority' }).click();
    await page.getByRole('textbox', { name: 'Priority' }).fill('B');
    await page.getByRole('textbox', { name: 'Description' }).click();
    await page.getByRole('textbox', { name: 'Description' }).fill(taskDescription);
    await page.getByRole('button', { name: 'Add' }).click();
  });

  test('can find the task again with the correct priority', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).toContain("(B)")
  });

  // Flagging as "done"
  test('can flag the task as done', async ({ page }) => {
    await page.getByTestId('task-flag-done-' + taskDescription).click();
  });

  test('does remember the task as done', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).not.toMatch(/\([A-Z]\)/) // Priority not shown
    expect(rowText).toContain("✗") // "Done" marker
  });

  // Flagging as "pending"
  test('can flag the task as pending', async ({ page }) => {
    await page.getByTestId('task-flag-pending-' + taskDescription).click();
  });

  test('does remember the task as pending', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).toContain("(B)") // Priority shown (and remembered)
    expect(rowText).not.toContain("✗") // "Done" marker
  });


  // Increasing priority
  test('can increase the priority', async ({ page }) => {
    await page.getByTestId('task-increase-priority-' + taskDescription).click();
  });

  test('does remember the priority as increased', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).toContain("(A)")
  });

  // Lowering priority
  test('can lower the priority', async ({ page }) => {
    await page.getByTestId('task-lower-priority-' + taskDescription).click();
  });

  test('does remember the priority as lowered', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription).innerText();

    expect(rowText).toContain(taskDescription)
    expect(rowText).toContain("(B)")
  });

  // Editing description
  test('can edit the description', async ({ page }) => {
    let locator = await page.getByRole('cell', { name: taskDescription });

    await locator.click();
    await locator.pressSequentially(" some more text");

    // TODO: find a more elegant way of simulating this in playwright
    await locator.dispatchEvent('focusout');
  });

  test('does remember new description', async ({ page }) => {
    let rowText = await page.getByTestId("task-row-" + taskDescription + " some more text").innerText();

    expect(rowText).toContain(taskDescription + " some more text")
  });
})