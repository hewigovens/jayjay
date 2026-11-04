import { expect, test } from "@playwright/test";
import { bundle, ready } from "./support";

test("retries against the newly imported change", async ({ page }) => {
  await page.addInitScript(() => {
    let lose = false;
    const lost = new Set<number>();
    (window as unknown as { loseNextMutation: () => void }).loseNextMutation = () => {
      lose = true;
    };
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string; id?: number }) {
        if (message.kind === "mutate" && lose) {
          lose = false;
          lost.add(message.id!);
        }
        super.postMessage(message);
      }
      set onmessage(handler: (event: MessageEvent) => void) {
        super.onmessage = (event) =>
          handler(
            lost.delete(event.data.id)
              ? new MessageEvent("message", { data: { id: event.data.id, error: "Response lost" } })
              : event,
          );
      }
    };
  });
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click();
  await ready(page);
  await bundle(page);
  await page.evaluate(() =>
    (window as unknown as { loseNextMutation: () => void }).loseNextMutation(),
  );
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Response lost");
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
});

test("keeps the file selected until an in-flight note save is acknowledged", async ({ page }) => {
  await page.addInitScript(() => {
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string }) {
        if (message.kind !== "mutate") return super.postMessage(message);
        (window as unknown as { releaseMutation: () => void }).releaseMutation = () =>
          super.postMessage(message);
      }
    };
  });
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Saved once");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await page.waitForFunction(() => "releaseMutation" in window);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click();
  await page.evaluate(() =>
    (window as unknown as { releaseMutation: () => void }).releaseMutation(),
  );
  await ready(page);
  await expect(page.getByRole("textbox", { name: "Write a review note", exact: true })).toHaveCount(
    0,
  );
  await expect(page.locator(".jj-file-heading h2")).toHaveText("src/greeting.rs");
  await expect(page.locator(".jj-review-note", { hasText: "Saved once" })).toHaveCount(1);
});

test("retrying after a lost save response re-renders instead of writing again", async ({
  page,
}) => {
  await page.addInitScript(() => {
    const counts = { mutate: 0 };
    (window as unknown as { counts: typeof counts }).counts = counts;
    const lost = new Set<number>();
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string; id?: number }) {
        if (message.kind === "mutate" && ++counts.mutate === 1) lost.add(message.id!);
        super.postMessage(message);
      }
      set onmessage(handler: (event: MessageEvent) => void) {
        super.onmessage = (event) =>
          handler(
            lost.delete(event.data.id)
              ? new MessageEvent("message", { data: { id: event.data.id, error: "Response lost" } })
              : event,
          );
      }
    };
  });
  await page.goto("/");
  await ready(page);
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Response lost");
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
  expect(
    await page.evaluate(() => (window as unknown as { counts: { mutate: number } }).counts.mutate),
  ).toBe(1);
});

test("serializes independent writes from two tabs", async ({ page, context }) => {
  await page.goto("/");
  await ready(page);
  const other = await context.newPage();
  await other.goto("/");
  await ready(other);
  await Promise.all([
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }).click(),
    other.getByRole("checkbox", { name: "Mark reviewed: src/web/session.ts", exact: true }).click(),
  ]);
  await ready(page);
  await ready(other);
  await page.reload();
  await ready(page);
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
  ).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: src/web/session.ts", exact: true }),
  ).toBeChecked();
});

test("does not acknowledge a failed browser-storage write", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.evaluate(
    () =>
      new Promise<void>((resolve, reject) => {
        const request = indexedDB.deleteDatabase("jayjay-review-tests");
        request.onsuccess = () => resolve();
        request.onerror = () => reject(request.error);
      }),
  );
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed", exact: true }),
  ).not.toBeChecked();
  await expect(page.locator(".jj-review-detail .jj-result-footer")).not.toContainText(
    "Saved on this device",
  );
});
