import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DeskView } from "./desk/DeskView";
import { CompactNotch } from "./CompactNotch";
import { desktop, windowJob } from "./desk/native";
import { useProviders } from "./desk/useProviders";

export default function App() {
  const providers = useProviders();
  const [mode, setMode] = useState<"desk" | "compact">("desk");
  const [busy, setBusy] = useState(false);
  const [pinned, setPinned] = useState(true);
  const [error, setError] = useState("");
  useEffect(() => {
    document.body.dataset.mode = mode;
  }, [mode]);
  const openDesk = useCallback(async () => {
    if (busy) return;
    setBusy(true);
    setError("");
    setMode("desk");
    try {
      if (desktop) await windowJob(() => invoke("show_desk"));
    } catch {
      setMode("compact");
      setError("Could not expand the window. Try again.");
    } finally {
      setBusy(false);
    }
  }, [busy]);
  useEffect(() => {
    if (!desktop) return;
    let alive = true;
    let stop: (() => void) | undefined;
    void listen("open-desk", () => {
      void openDesk();
    })
      .then((unlisten) => {
        if (alive) stop = unlisten;
        else unlisten();
      })
      .catch(() => {
        if (alive)
          setError(
            "Could not connect the system tray. Reopen the app to retry.",
          );
      });
    return () => {
      alive = false;
      stop?.();
    };
  }, [openDesk]);
  return mode === "desk" ? (
    <DeskView
      providers={providers}
      pinned={pinned}
      setPinned={setPinned}
      busy={busy}
      windowError={error}
      onCompact={() => {
        setError("");
        setMode("compact");
      }}
    />
  ) : (
    <CompactNotch
      providers={providers}
      onOpenDesk={() => void openDesk()}
      busy={busy}
      error={error}
    />
  );
}
