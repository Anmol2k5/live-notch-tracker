import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { NotchRootView } from './components/NotchRootView';
import { anchorNotch, BODY_DEPTH_PT, panelHeightForCells, type WorkArea } from './geometry/notchGeometry';
import { useUsageStore } from './state/usageStore';

// Format the provider data for the UI
function toFixtureCells(providers: ReturnType<typeof useUsageStore>['providers']) {
  return providers.map(({ id, snapshot }) => {
    // Determine the most restrictive window's percentage
    let percent = 0;
    if (snapshot.windows.length > 0) {
      // Find max usage across all windows
      const maxUsed = Math.max(...snapshot.windows.map(w => w.used));
      percent = Math.round(maxUsed * 100);
    }
    
    // Antigravity (derived count) is handled by looking at count, but UI handles percentages
    if (snapshot.windows.length === 1 && snapshot.windows[0].derived) {
      percent = snapshot.windows[0].count ?? 0;
    }

    return {
      id,
      initial: id.charAt(0).toUpperCase(),
      percent,
      status: snapshot.status
    };
  });
}

export default function App() {
  const { providers } = useUsageStore();
  const cells = toFixtureCells(providers);
  const [rect, setRect] = useState({ x: 0, y: 0, width: BODY_DEPTH_PT, height: panelHeightForCells(3) });

  useEffect(() => {
    let cancelled = false;
    async function place(): Promise<void> {
      try {
        const work = await invoke<WorkArea>('get_work_area');
        if (cancelled) return;
        
        // We anchor based on the number of active cells (minimum 1 so the notch doesn't vanish completely)
        const cellCount = Math.max(1, cells.length);
        const next = anchorNotch(work, cellCount);
        
        setRect(next);
        const win = getCurrentWindow();
        await win.setSize(new LogicalSize(next.width, next.height));
        await win.setPosition(new LogicalPosition(next.x, next.y));
        await win.show();
        // Re-apply Win32 exstyles after show(): Tauri may reset them on show().
        // This makes the Alt-Tab exclusion and click-through stick in the
        // production build (verified with WindowFromPoint + exstyle checks).
        try { await invoke('apply_notch_styles_cmd'); } catch { /* non-Windows or early */ }
        // M6a: make the window itself the silhouette shape via SetWindowRgn.
        // This makes the transparent margin pass through while the black notch
        // stays interactive (per-pixel, not the old whole-window TRANSPARENT).
        // Re-applied on every resize / DPI change via the cells.length dep.
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
    }
    void place();
    return () => {
      cancelled = true;
    };
  }, [cells.length]);

  return (
    <div style={{ width: rect.width, height: rect.height }}>
      <NotchRootView cells={cells} width={rect.width} height={rect.height} />
    </div>
  );
}
