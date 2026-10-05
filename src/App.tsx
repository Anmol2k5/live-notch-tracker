import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { NotchRootView, UIProviderCell } from './components/NotchRootView';
import { anchorNotch, BODY_DEPTH_PT, panelHeightForCells, type WorkArea } from './geometry/notchGeometry';
import { useUsageStore } from './state/usageStore';
import { resolveMetric } from './model/providerTypes';

// Format the provider data for the UI with explicit domain metrics
function toCells(providers: ReturnType<typeof useUsageStore>['providers']): UIProviderCell[] {
  return providers.map(({ id, snapshot }) => {
    let metric;
    if (snapshot.windows.length > 0) {
      // Primary window is first window (or session for Claude)
      const primary = snapshot.windows.find((w) => w.id === 'session') || snapshot.windows[0];
      metric = resolveMetric(primary);
    } else {
      metric = { kind: 'percentage' as const, usedPercent: 0 };
    }

    return {
      id,
      metric,
      status: snapshot.status,
      snapshot,
    };
  });
}

export default function App() {
  const { providers, refreshAll } = useUsageStore();
  const cells = toCells(providers);
  const [rect, setRect] = useState({ x: 0, y: 0, width: BODY_DEPTH_PT, height: panelHeightForCells(3) });

  const place = useCallback(async () => {
    try {
      const work = await invoke<WorkArea>('get_work_area');
      const cellCount = Math.max(1, cells.length);
      const next = anchorNotch(work, cellCount);

      setRect(next);
      const win = getCurrentWindow();
      await win.setSize(new LogicalSize(next.width, next.height));
      await win.setPosition(new LogicalPosition(next.x, next.y));
      await win.show();

      // Re-apply Win32 exstyles after show()
      try {
        await invoke('apply_notch_styles_cmd');
      } catch {
        /* non-Windows or early */
      }

      // Re-apply Win32 region with authoritative physical scaling
      try {
        await invoke('apply_notch_region_cmd', {
          width: next.width,
          height: next.height,
          scale_factor: work.scaleFactor,
          scaleFactor: work.scaleFactor,
        });
      } catch (e) {
        console.error('[codenotch] region failed:', e);
      }
    } catch (err) {
      console.error('[codenotch] placement failed:', err);
    }
  }, [cells.length]);

  useEffect(() => {
    void place();
  }, [place]);

  // Listen for DPI scale changes and monitor moves
  useEffect(() => {
    let unlistenScale: (() => void) | undefined;
    let unlistenMove: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        const win = getCurrentWindow();
        unlistenScale = await win.onScaleChanged(() => {
          void place();
        });
        unlistenMove = await win.onMoved(() => {
          void place();
        });
      } catch (e) {
        console.warn('[codenotch] listener setup warning:', e);
      }
    };

    void setupListeners();

    return () => {
      if (unlistenScale) unlistenScale();
      if (unlistenMove) unlistenMove();
    };
  }, [place]);

  return (
    <div style={{ width: rect.width, height: rect.height }}>
      <NotchRootView
        cells={cells}
        width={rect.width}
        height={rect.height}
        onRefreshAll={refreshAll}
      />
    </div>
  );
}
