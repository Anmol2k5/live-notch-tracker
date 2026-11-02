import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { desktop } from "./native";
import { Icon } from "./Icon";
import { TasksView } from "./TasksView";
import { UsageView } from "./UsageView";
import { GitHubView } from "./GitHubView";
import { QuoteView } from "./QuoteView";
import { AppsView } from "./AppsView";
import { StartupToggle } from "./StartupToggle";
import { useProviders } from "./useProviders";
import "./desk.css";

export function DeskView({
  providers,
  onCompact,
  busy,
  windowError,
  pinned,
  setPinned,
}: {
  providers: ReturnType<typeof useProviders>;
  onCompact: () => void;
  busy: boolean;
  windowError: string;
  pinned: boolean;
  setPinned: (pinned: boolean) => void;
}) {
  const [error, setError] = useState("");
  return (
    <main className={`desk-shell ${desktop ? "native-desk" : "browser-desk"}`}>
      <header className="desk-header" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-mark">
            <span />
            <span />
            <span />
          </span>
          <span>
            codenotch<span className="brand-dot">.</span>
          </span>
          <span className="edition">FOR WINDOWS</span>
        </div>
        <div className="window-actions">
          <button
            className={`icon-button ${pinned && desktop ? "active" : ""}`}
            aria-label={pinned ? "Unpin window" : "Pin window"}
            aria-pressed={pinned}
            disabled={!desktop}
            title="Keep on top"
            onClick={async () => {
              try {
                await getCurrentWindow().setAlwaysOnTop(!pinned);
                setPinned(!pinned);
              } catch {
                setError("Could not change the window pin.");
              }
            }}
          >
            <Icon name="pin" size={15} />
          </button>
          <button
            className="compact-button"
            onClick={onCompact}
            disabled={busy}
          >
            <Icon name="compact" size={15} /> Compact notch
          </button>
          {desktop && (
            <button
              className="icon-button close-window"
              aria-label="Hide Codenotch to tray"
              title="Hide to system tray. Use the tray menu to quit."
              onClick={() => {
                void getCurrentWindow()
                  .close()
                  .catch(() => setError("Could not close the window."));
              }}
            >
              <Icon name="close" size={16} />
            </button>
          )}
        </div>
      </header>
      <div className="desk-content">
        <div className="desk-intro">
          <div>
            <div className="eyebrow">
              <span className="status-dot" /> YOUR PERSONAL DESK
            </div>
            <h1>
              A little space to focus<span>.</span>
            </h1>
            <p>Your tasks, your tools, and a little inspiration.</p>
          </div>
          <div className="desk-date">
            <span>
              {new Date().toLocaleDateString(undefined, { weekday: "long" })}
            </span>
            <strong>
              {new Date().toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
              })}
            </strong>
          </div>
        </div>
        {(error || windowError) && (
          <p className="inline-error window-error" role="alert">
            {error || windowError}
          </p>
        )}
        <div className="desk-grid">
          <TasksView />
          <UsageView data={providers} />
          <GitHubView />
          <QuoteView />
          <AppsView />
        </div>
        <footer className="desk-footer">
          <StartupToggle />
          <span>
            <span className="tiny-lock" aria-hidden="true">
              ◇
            </span>{" "}
            A quieter corner of your desktop.
          </span>
          <span>
            {desktop ? "WINDOWS DESK" : "BROWSER VIEW"}
            <i /> CODENOTCH 0.1
          </span>
        </footer>
      </div>
    </main>
  );
}
