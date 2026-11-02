import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { UsageDetails } from "./UsageView";
import { parseUsageSnapshot } from "../model/providerTypes";

const snapshot = (status: string, windows: unknown[] = []) =>
  parseUsageSnapshot({
    status,
    windows,
    note: "",
    fetched_at: 1,
    backoff_until: 0,
  } as Parameters<typeof parseUsageSnapshot>[0]);
it("shows unavailable data without fabricating a zero-percent limit", () => {
  const html = renderToStaticMarkup(
    <UsageDetails snapshot={snapshot("needsAuth")} loading={false} />,
  );
  expect(html).toContain("Sign in");
  expect(html).not.toContain("0%");
});
it("identifies stale readings and preserves count-only metrics", () => {
  const html = renderToStaticMarkup(
    <UsageDetails
      loading={false}
      snapshot={snapshot("stale", [
        {
          id: "today",
          label: "Requests today",
          count: 42,
          used: 0,
          resets_at: null,
        },
      ])}
    />,
  );
  expect(html).toContain("Saved reading");
  expect(html).toContain("42");
  expect(html).not.toContain("42%");
  expect(html).not.toContain('role="progressbar"');
});
it("shows a bounded percentage and accessible progress only for available limits", () => {
  const html = renderToStaticMarkup(
    <UsageDetails
      loading={false}
      snapshot={snapshot("ok", [
        { id: "session", label: "5-hour limit", used: 0.35, resets_at: null },
      ])}
    />,
  );
  expect(html).toContain("35%");
  expect(html).toContain('aria-valuenow="35"');
  expect(html).toContain("65% left");
});
