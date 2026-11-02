import { useState } from "react";
import { AppId, launchApp } from "./native";
import { Icon } from "./Icon";

const apps: { id: AppId; label: string; color: string }[] = [
  { id: "chrome", label: "Chrome", color: "#e9b559" },
  { id: "discord", label: "Discord", color: "#8b8df8" },
  { id: "spotify", label: "Spotify", color: "#50d88b" },
  { id: "obs", label: "OBS Studio", color: "#d7ddda" },
];
function AppMark({ id }: { id: AppId }) {
  if (id === "chrome")
    return (
      <svg viewBox="0 0 40 40" aria-hidden="true">
        <circle cx="20" cy="20" r="18" fill="#f3c44e" />
        <path d="M20 20 5 10a18 18 0 0 1 31 2H20Z" fill="#ec6659" />
        <path d="m20 20-9 16A18 18 0 0 1 5 10l8 14Z" fill="#60b881" />
        <circle
          cx="20"
          cy="20"
          r="8"
          fill="#66b1e3"
          stroke="#edf6ef"
          strokeWidth="2"
        />
      </svg>
    );
  if (id === "discord")
    return (
      <svg viewBox="0 0 40 40" fill="#8b8df8" aria-hidden="true">
        <path d="m10 9 7-2 1 3h4l1-3 7 2c5 7 7 14 6 19l-9 4-2-4 3-2c-5 2-11 2-16 0l3 2-2 4-9-4C3 23 5 16 10 9Z" />
        <ellipse cx="14" cy="20" rx="2.5" ry="3" fill="#141817" />
        <ellipse cx="26" cy="20" rx="2.5" ry="3" fill="#141817" />
      </svg>
    );
  if (id === "spotify")
    return (
      <svg viewBox="0 0 40 40" aria-hidden="true">
        <circle cx="20" cy="20" r="18" fill="#4bd58b" />
        <g fill="none" stroke="#11241a" strokeLinecap="round">
          <path d="M9 15c8-3 17-2 23 1" strokeWidth="3" />
          <path d="M11 21c7-2 13-1 19 2" strokeWidth="2.6" />
          <path d="M13 27c5-1 10-1 15 1" strokeWidth="2.3" />
        </g>
      </svg>
    );
  return (
    <svg viewBox="0 0 40 40" aria-hidden="true">
      <circle
        cx="20"
        cy="20"
        r="17"
        stroke="#d7ddda"
        fill="#1a1e1c"
        strokeWidth="1.5"
      />
      <g fill="none" stroke="#d7ddda" strokeWidth="3">
        <path d="M20 5c-7 4-5 13 2 15 M34 26c0-8-9-11-14-6 M7 28c7 4 14-2 13-9" />
      </g>
    </svg>
  );
}
export function AppsView() {
  const [opening, setOpening] = useState<AppId | null>(null);
  const [message, setMessage] = useState("");
  return (
    <section className="widget apps-widget" aria-labelledby="apps-title">
      <div className="widget-heading">
        <h2 id="apps-title">
          <Icon name="apps" /> Within reach
        </h2>
        <span className="tag">4 apps</span>
      </div>
      <div className="app-grid">
        {apps.map((app) => (
          <button
            className="app-shortcut"
            key={app.id}
            aria-label={`Open ${app.label}`}
            disabled={opening !== null}
            onClick={async () => {
              setOpening(app.id);
              setMessage("");
              try {
                await launchApp(app.id);
                setMessage(`${app.label} launched.`);
              } catch (e) {
                setMessage(String(e instanceof Error ? e.message : e));
              } finally {
                setOpening(null);
              }
            }}
          >
            <span
              className="app-icon"
              style={{ "--app-color": app.color } as React.CSSProperties}
            >
              <AppMark id={app.id} />
            </span>
            <span>{opening === app.id ? "Opening…" : app.label}</span>
          </button>
        ))}
      </div>
      <div className="widget-foot" role="status">
        {message || "Your everyday apps. One click away."}
      </div>
    </section>
  );
}
