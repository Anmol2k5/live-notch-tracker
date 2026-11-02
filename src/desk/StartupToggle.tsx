import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { desktop } from "./native";

export function StartupToggle() {
  const [enabled, setEnabled] = useState(false);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!desktop) return;
    let alive = true;
    void invoke<boolean>("get_startup_enabled")
      .then((value) => {
        if (alive) setEnabled(value);
      })
      .catch(() => {
        if (alive) setError("Could not read Windows startup settings.");
      })
      .finally(() => {
        if (alive) setBusy(false);
      });
    return () => {
      alive = false;
    };
  }, []);
  if (!desktop) return null;
  return (
    <div className="startup-setting">
      <button
        className="startup-switch"
        role="switch"
        aria-label="Start with Windows"
        aria-checked={enabled}
        disabled={busy}
        onClick={async () => {
          setBusy(true);
          setError("");
          try {
            setEnabled(
              await invoke<boolean>("set_startup_enabled", {
                enabled: !enabled,
              }),
            );
          } catch (e) {
            setError(String(e instanceof Error ? e.message : e));
          } finally {
            setBusy(false);
          }
        }}
      >
        <span className={enabled ? "switch-track enabled" : "switch-track"}>
          <i />
        </span>
        Start with Windows
      </button>
      {error && (
        <span className="inline-error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
