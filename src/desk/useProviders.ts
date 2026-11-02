import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  parseUsageSnapshot,
  ProviderId,
  RawUsageSnapshot,
  UsageSnapshot,
} from "../model/providerTypes";
import { desktop } from "./native";

export const PROVIDERS: ProviderId[] = [
  "codex",
  "claude",
  "cursor",
  "antigravity",
];
const commands = {
  claude: "get_usage",
  codex: "get_codex",
  cursor: "get_cursor",
  antigravity: "get_antigravity",
};
const events = {
  claude: "usage",
  codex: "codex",
  cursor: "cursor",
  antigravity: "antigravity",
};
export function useProviders() {
  const [snapshots, setSnapshots] = useState<
    Partial<Record<ProviderId, UsageSnapshot>>
  >({});
  const [loading, setLoading] = useState(desktop);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!desktop) return;
    let alive = true;
    const stops: UnlistenFn[] = [];
    const revisions: Partial<Record<ProviderId, number>> = {};
    const accept = (id: ProviderId, raw: RawUsageSnapshot) => {
      if (alive) setSnapshots((s) => ({ ...s, [id]: parseUsageSnapshot(raw) }));
    };
    void Promise.all(
      PROVIDERS.map(async (id) => {
        try {
          const stop = await listen<RawUsageSnapshot>(events[id], (event) => {
            revisions[id] = (revisions[id] ?? 0) + 1;
            accept(id, event.payload);
          });
          if (!alive) {
            stop();
            return;
          }
          stops.push(stop);
          const revision = revisions[id] ?? 0;
          const raw = await invoke<RawUsageSnapshot>(commands[id]);
          if ((revisions[id] ?? 0) === revision) accept(id, raw);
        } catch {
          if (alive)
            setError("Could not read local provider data. Try refreshing.");
        }
      }),
    ).finally(() => {
      if (alive) setLoading(false);
    });
    return () => {
      alive = false;
      stops.forEach((stop) => stop());
    };
  }, []);
  const refresh = useCallback(async () => {
    if (!desktop) return;
    try {
      await invoke("refresh_usage");
      setError("");
    } catch {
      setError("Could not request a refresh. Try reopening Codenotch.");
    }
  }, []);
  return { snapshots, loading, error, refresh };
}
