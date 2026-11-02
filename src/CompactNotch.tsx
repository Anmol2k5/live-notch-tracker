import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { NotchRootView, UIProviderCell } from "./components/NotchRootView";
import {
  anchorNotch,
  BODY_DEPTH_PT,
  panelHeightForCells,
  WorkArea,
} from "./geometry/notchGeometry";
import { resolveMetric } from "./model/providerTypes";
import { desktop, windowJob } from "./desk/native";
import { PROVIDERS, useProviders } from "./desk/useProviders";

export function CompactNotch({
  providers,
  onOpenDesk,
  busy,
  error,
}: {
  providers: ReturnType<typeof useProviders>;
  onOpenDesk: () => void;
  busy: boolean;
  error: string;
}) {
  const cells: UIProviderCell[] = PROVIDERS.flatMap((id) => {
    const snapshot = providers.snapshots[id];
    if (
      !snapshot?.windows.length ||
      ["none", "absent"].includes(snapshot.status.kind)
    )
      return [];
    const primary =
      snapshot.windows.find((w) => w.id === "session") || snapshot.windows[0];
    return [
      { id, snapshot, status: snapshot.status, metric: resolveMetric(primary) },
    ];
  });
  const height = panelHeightForCells(Math.max(cells.length, 1)) + 74;
  const [rect, setRect] = useState({ width: BODY_DEPTH_PT, height });
  const [placementError, setPlacementError] = useState("");
  useEffect(() => {
    if (!desktop) {
      setRect({ width: BODY_DEPTH_PT, height });
      return;
    }
    let alive = true;
    let stop: (() => void) | undefined;
    const place = () => {
      void windowJob(async () => {
        if (!alive) return;
        const work = await invoke<WorkArea>("get_work_area");
        if (!alive) return;
        const next = anchorNotch(work, Math.max(cells.length, 1));
        next.height = height;
        next.y -= 37;
        const win = getCurrentWindow();
        await win.setSize(new LogicalSize(next.width, next.height));
        if (!alive) return;
        await win.setPosition(new LogicalPosition(next.x, next.y));
        await invoke("apply_notch_styles_cmd");
        if (!alive) return;
        await invoke("apply_notch_region_cmd", {
          width: next.width,
          height: next.height,
          scaleFactor: work.scaleFactor,
        });
        if (alive) {
          setRect(next);
        }
      }).catch(() => {
        if (alive)
          setPlacementError("Window placement failed. Open Desk to retry.");
      });
    };
    place();
    void getCurrentWindow()
      .onScaleChanged(place)
      .then((unlisten) => {
        if (alive) stop = unlisten;
        else unlisten();
      });
    return () => {
      alive = false;
      stop?.();
    };
  }, [cells.length, height]);
  return (
    <div className="compact-container" style={rect}>
      <NotchRootView
        cells={cells}
        width={rect.width}
        height={rect.height}
        onRefreshAll={() => void providers.refresh()}
        onOpenDesk={onOpenDesk}
      />
      {(error || placementError || busy) && (
        <span
          className="compact-message"
          role="status"
          title={error || placementError}
        >
          {busy ? "Opening…" : "Retry Desk"}
        </span>
      )}
    </div>
  );
}
