import { test, expect, Page } from "@playwright/test";

// Fake only the native IPC boundary; the real React/window APIs and queue still run.
async function nativeHarness(page: Page, failExpansion: boolean) {
  await page.addInitScript(
    ({ failExpansion }) => {
      let callback = 0;
      let attempts = 0;
      Object.assign(window, {
        isTauri: true,
        __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
        __TAURI_INTERNALS__: {
          metadata: {
            currentWindow: { label: "notch" },
            currentWebview: { label: "notch" },
          },
          transformCallback: () => ++callback,
          invoke: async (command: string) => {
            if (command === "show_desk" && failExpansion && ++attempts === 1)
              throw new Error("Window resize failed");
            if (command === "get_work_area")
              return {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040,
                scaleFactor: 1.25,
              };
            if (command.startsWith("get_"))
              return {
                status: "absent",
                windows: [],
                fetched_at: 0,
                note: "",
                backoff_until: 0,
              };
            if (command === "plugin:event|listen") return ++callback;
            return null;
          },
        },
      });
    },
    { failExpansion },
  );
}

test("project tasks persist across reloads and stay isolated by project", async ({
  page,
}) => {
  await page.goto("/");
  await page
    .getByRole("textbox", { name: "New task" })
    .fill("Ship the Windows desk");
  await page.getByRole("button", { name: "Add task" }).click();
  const done = page.getByRole("checkbox", {
    name: "Complete Ship the Windows desk",
  });
  await done.click();
  await expect(done).toHaveAttribute("aria-checked", "true");
  await page.reload();
  await expect(done).toHaveAttribute("aria-checked", "true");
  await page.getByRole("button", { name: "Add project", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Project name" })
    .fill("Windows release");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(done).toHaveCount(0);
  await page.getByRole("textbox", { name: "New task" }).fill("Check DPI");
  await page.getByRole("button", { name: "Add task" }).click();
  await page.getByRole("button", { name: "Personal", exact: true }).click();
  await expect(done).toHaveCount(1);
  await expect(
    page.getByRole("checkbox", { name: "Complete Check DPI" }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "Windows release", exact: true })
    .click();
  await page.getByRole("button", { name: "Delete current project" }).click();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    page.getByRole("checkbox", { name: "Complete Check DPI" }),
  ).toHaveCount(1);
  await page.getByRole("button", { name: "Delete current project" }).click();
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  await expect(done).toHaveCount(1);
  await page
    .getByRole("button", { name: "Delete Ship the Windows desk", exact: true })
    .click();
  await expect(done).toHaveCount(0);
});

test("widgets work in a browser and explain native-only actions without uncaught errors", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await page.getByRole("button", { name: "Next quote" }).click();
  await expect(
    page.getByText("It always seems impossible until it’s done.", {
      exact: true,
    }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Claude Code", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Claude Code", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Open Chrome", exact: true }).click();
  await expect(page.getByRole("status")).toContainText(
    "App shortcuts work in the Windows desktop app.",
  );
  await page.getByRole("textbox", { name: "GitHub username" }).fill("Anmol2k5");
  await page.getByRole("button", { name: "Load GitHub activity" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Open the Windows app to load your GitHub activity.",
  );
  await page.getByRole("button", { name: "Compact notch" }).click();
  await page.getByRole("button", { name: "Open Desk" }).click();
  await expect(
    page.getByRole("heading", { name: "Your next little wins" }),
  ).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  expect(errors).toEqual([]);
});

test("corrupt saved data stays untouched until the user explicitly resets it", async ({
  page,
}) => {
  await page.goto("/");
  await page.evaluate(() =>
    localStorage.setItem("codenotch.desk.v1", "{broken"),
  );
  await page.reload();
  await expect(page.getByRole("alert")).toContainText(
    "saved tasks could not be read",
  );
  await expect(page.getByRole("textbox", { name: "New task" })).toBeDisabled();
  expect(
    await page.evaluate(() => localStorage.getItem("codenotch.desk.v1")),
  ).toBe("{broken");
  await page
    .getByRole("button", { name: "Discard unreadable data & start fresh" })
    .click();
  await expect(page.getByRole("textbox", { name: "New task" })).toBeEnabled();
});

test("a storage failure never appears as a successfully saved task", async ({
  page,
}) => {
  await page.goto("/");
  await page.evaluate(() => {
    Storage.prototype.setItem = () => {
      throw new DOMException("Storage is full", "QuotaExceededError");
    };
  });
  await page.getByRole("textbox", { name: "New task" }).fill("Unsaved task");
  await page.getByRole("button", { name: "Add task" }).click();
  await expect(page.getByRole("alert")).toContainText("Storage is full");
  await expect(
    page.getByRole("checkbox", { name: "Complete Unsaved task" }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(() => localStorage.getItem("codenotch.desk.v1")),
  ).toBeNull();
});

test("the entire Desk return button fits inside the compact native silhouette", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Compact notch" }).click();
  const inside = await page
    .getByRole("button", { name: "Open Desk" })
    .evaluate((button) => {
      const svg = document.querySelector(
        ".compact-container svg",
      ) as SVGSVGElement;
      const path = svg.querySelector("path") as SVGGeometryElement;
      const origin = svg.getBoundingClientRect();
      const rect = button.getBoundingClientRect();
      return [
        [rect.left, rect.top],
        [rect.right, rect.top],
        [rect.left, rect.bottom],
        [rect.right, rect.bottom],
      ].every(([x, y]) =>
        path.isPointInFill(new DOMPoint(x - origin.left, y - origin.top)),
      );
    });
  expect(inside).toBe(true);
});

test("failed native expansion keeps a reachable compact retry control", async ({
  page,
}) => {
  await nativeHarness(page, true);
  await page.goto("/");
  await page.getByRole("button", { name: "Compact notch" }).click();
  await page.getByRole("button", { name: "Open Desk" }).click();
  await expect(page.getByRole("button", { name: "Open Desk" })).toBeVisible();
  await page.getByRole("button", { name: "Open Desk" }).click();
  await expect(
    page.getByRole("heading", { name: "Your next little wins" }),
  ).toBeVisible();
});

test("window pin state survives a compact roundtrip", async ({ page }) => {
  await nativeHarness(page, false);
  await page.goto("/");
  await page.getByRole("button", { name: "Unpin window" }).click();
  await expect(
    page.getByRole("button", { name: "Pin window" }),
  ).toHaveAttribute("aria-pressed", "false");
  await page.getByRole("button", { name: "Compact notch" }).click();
  await page.getByRole("button", { name: "Open Desk" }).click();
  await expect(
    page.getByRole("button", { name: "Pin window" }),
  ).toHaveAttribute("aria-pressed", "false");
});
