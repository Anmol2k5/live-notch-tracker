import { test, expect, Page } from "@playwright/test";

async function startupHarness(page: Page) {
  await page.addInitScript(() => {
    let callback = 0;
    const commands: string[] = [];
    const listeners = new Map<string, number>();
    const handlers = new Map<number, (event: unknown) => void>();
    Object.assign(window, {
      isTauri: true,
      __nativeCommands: commands,
      __nativeEmit: (event: string, payload: unknown) =>
        handlers.get(listeners.get(event) ?? -1)?.({ event, id: 1, payload }),
      __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
      __nativeOpenDesk: () =>
        handlers.get(listeners.get("open-desk") ?? -1)?.({
          event: "open-desk",
          id: 1,
          payload: null,
        }),
      __TAURI_INTERNALS__: {
        metadata: {
          currentWindow: { label: "notch" },
          currentWebview: { label: "notch" },
        },
        transformCallback: (handler: (event: unknown) => void) => {
          handlers.set(++callback, handler);
          return callback;
        },
        invoke: async (
          command: string,
          args: { event?: string; handler?: number; enabled?: boolean },
        ) => {
          commands.push(command);
          if (command === "get_startup_enabled") return false;
          if (command === "set_startup_enabled")
            throw new Error("Windows denied the startup change.");
          if (command === "plugin:event|listen") {
            listeners.set(args.event!, args.handler!);
            return ++callback;
          }
          if (command === "get_work_area")
            return { x: 0, y: 0, width: 1920, height: 1040, scaleFactor: 1 };
          if (command.startsWith("get_"))
            return {
              status: "absent",
              windows: [],
              fetched_at: 0,
              note: "",
              backoff_until: 0,
            };
          return null;
        },
      },
    });
  });
}

test("startup is opt-in and a failed native save leaves the switch unchanged", async ({
  page,
}) => {
  await startupHarness(page);
  await page.goto("/");
  const startup = page.getByRole("switch", { name: "Start with Windows" });
  await expect(startup).toHaveAttribute("aria-checked", "false");
  await startup.click();
  await expect(page.getByRole("alert")).toContainText(
    "Windows denied the startup change.",
  );
  await expect(startup).toHaveAttribute("aria-checked", "false");
  await page.getByRole("button", { name: "Compact notch" }).click();
  await page.evaluate(() =>
    (window as unknown as { __nativeOpenDesk: () => void }).__nativeOpenDesk(),
  );
  await expect(
    page.getByRole("heading", { name: "Your next little wins" }),
  ).toBeVisible();
});

test("compact placement never unhides a window after a provider event", async ({
  page,
}) => {
  await startupHarness(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Compact notch" }).click();
  type Harness = {
    __nativeCommands: string[];
    __nativeEmit: (event: string, payload: unknown) => void;
  };
  await expect
    .poll(() =>
      page.evaluate(() =>
        (
          window as unknown as { __nativeCommands: string[] }
        ).__nativeCommands.includes("apply_notch_region_cmd"),
      ),
    )
    .toBe(true);
  await page.evaluate(() => {
    const native = window as unknown as Harness;
    native.__nativeCommands.length = 0;
    native.__nativeEmit("codex", {
      status: "ok",
      windows: [
        { id: "session", label: "5-hour limit", used: 0.4, resets_at: null },
      ],
      fetched_at: Date.now(),
      note: "",
      backoff_until: 0,
    });
  });
  await expect
    .poll(() =>
      page.evaluate(() =>
        (
          window as unknown as { __nativeCommands: string[] }
        ).__nativeCommands.includes("apply_notch_region_cmd"),
      ),
    )
    .toBe(true);
  expect(
    await page.evaluate(() =>
      (
        window as unknown as { __nativeCommands: string[] }
      ).__nativeCommands.includes("plugin:window|show"),
    ),
  ).toBe(false);
});
