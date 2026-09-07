import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { NotchRootView } from './components/NotchRootView';
import { anchorNotch, BODY_DEPTH_PT, panelHeightForCells, type WorkArea } from './geometry/notchGeometry';
import { fixtures } from './state/fixtures';

export default function App() {
  const [rect, setRect] = useState({ x: 0, y: 0, width: BODY_DEPTH_PT, height: panelHeightForCells(3) });

  useEffect(() => {
    let cancelled = false;
    async function place(): Promise<void> {
      const work = await invoke<WorkArea>('get_work_area');
      if (cancelled) {
        return;
      }
      const next = anchorNotch(work, fixtures.length);
      setRect(next);
      const win = getCurrentWindow();
      await win.setSize(new LogicalSize(next.width, next.height));
      await win.setPosition(new LogicalPosition(next.x, next.y));
    }
    void place();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div style={{ width: rect.width, height: rect.height }}>
      <NotchRootView cells={fixtures} width={rect.width} height={rect.height} />
    </div>
  );
}
