import { useState } from "react";
import {
  formatMetricLabel,
  metricRingFraction,
  PROVIDER_METADATA,
  ProviderId,
  resolveMetric,
  UsageSnapshot,
} from "../model/providerTypes";
import { ProviderMark } from "../components/ProviderMark";
import { Icon } from "./Icon";
import { desktop } from "./native";
import { PROVIDERS, useProviders } from "./useProviders";

const statuses: Record<string, string> = {
  absent:
    "No local session found. Open this provider on your PC to get started.",
  none: "No usage reading yet. Open this provider and try refreshing.",
  needsAuth: "Sign in to this provider on your PC to read its usage.",
  error: "The provider could not be read. Try refreshing in a moment.",
};
export function UsageDetails({
  snapshot,
  loading,
}: {
  snapshot?: UsageSnapshot;
  loading: boolean;
}) {
  if (loading && !snapshot)
    return <div className="usage-empty">Checking local sessions…</div>;
  if (
    !snapshot ||
    ["absent", "none", "needsAuth", "error"].includes(snapshot.status.kind) ||
    !snapshot.windows.length
  ) {
    return (
      <div className="usage-empty">
        <span className="empty-orbit">
          <Icon name="activity" size={24} />
        </span>
        <p>
          {!desktop && !snapshot
            ? "Your agents, in one glance."
            : "Waiting for a reading"}
        </p>
        <span>
          {snapshot
            ? statuses[snapshot.status.kind] ||
              "No limits published for this session yet."
            : "Open Codenotch on Windows to read your local AI sessions."}
        </span>
      </div>
    );
  }
  const stale =
    snapshot.status.kind === "stale" || snapshot.status.kind === "backoff";
  return (
    <div className={`usage-details ${stale ? "is-stale" : ""}`}>
      {stale && (
        <p className="reading-status">
          Saved reading ·{" "}
          {snapshot.status.kind === "backoff"
            ? "waiting before retry"
            : "refresh pending"}
        </p>
      )}
      {snapshot.windows.map((window) => {
        const metric = resolveMetric(window);
        const fraction = metricRingFraction(metric);
        const percent = fraction === null ? null : Math.round(fraction * 100);
        const reset = metric.resetsAt;
        return (
          <div className="usage-limit" key={window.id}>
            <div className="limit-heading">
              <span>{window.label}</span>
              <strong>{formatMetricLabel(metric)}</strong>
            </div>
            {percent !== null && (
              <div
                className="usage-track"
                role="progressbar"
                aria-label={window.label}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={percent}
              >
                <span
                  style={{
                    width: `${percent}%`,
                    background: percent >= 90 ? "#f3ab77" : undefined,
                  }}
                />
              </div>
            )}
            <div className="limit-caption">
              <span>
                {percent === null
                  ? "No published limit"
                  : `${100 - percent}% left`}
              </span>
              <span>
                {reset
                  ? `Resets ${new Date(reset).toLocaleString(undefined, { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" })}`
                  : "Reset time unavailable"}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
export function UsageView({ data }: { data: ReturnType<typeof useProviders> }) {
  const [selected, setSelected] = useState<ProviderId>("codex");
  const [refreshing, setRefreshing] = useState(false);
  const snapshot = data.snapshots[selected];
  return (
    <section className="widget usage-widget" aria-labelledby="usage-title">
      <div className="widget-heading">
        <h2 id="usage-title">
          <Icon name="activity" /> AI usage
        </h2>
        <button
          className={`icon-button ${refreshing ? "spinning" : ""}`}
          aria-label="Refresh AI usage"
          disabled={!desktop || refreshing}
          title={
            desktop ? "Refresh local sessions" : "Available in the Windows app"
          }
          onClick={async () => {
            setRefreshing(true);
            await data.refresh();
            setRefreshing(false);
          }}
        >
          <Icon name="refresh" size={16} />
        </button>
      </div>
      <div className="provider-tabs" role="group" aria-label="AI provider">
        {PROVIDERS.map((id) => (
          <button
            key={id}
            className={`provider-tab ${selected === id ? "selected" : ""}`}
            aria-pressed={selected === id}
            aria-label={PROVIDER_METADATA[id].displayName}
            onClick={() => setSelected(id)}
          >
            <ProviderMark id={id} size={21} />
          </button>
        ))}
        <span>{PROVIDER_METADATA[selected].displayName}</span>
      </div>
      <UsageDetails snapshot={snapshot} loading={data.loading} />
      {data.error && (
        <p className="inline-error" role="alert">
          {data.error}
        </p>
      )}
      <div className="widget-foot">
        <span
          className={`status-dot ${snapshot?.status.kind === "ok" ? "" : "muted"}`}
        />
        {snapshot?.fetched_at
          ? `Updated ${new Date(snapshot.fetched_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`
          : "Local sessions · read only"}
      </div>
    </section>
  );
}
