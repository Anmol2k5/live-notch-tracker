import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { RawUsageSnapshot, UsageSnapshot, parseUsageSnapshot } from '../model/providerTypes';

export function useUsageStore() {
  const [claude, setClaude] = useState<UsageSnapshot | null>(null);
  const [cursor, setCursor] = useState<UsageSnapshot | null>(null);
  const [codex, setCodex] = useState<UsageSnapshot | null>(null);
  const [antigravity, setAntigravity] = useState<UsageSnapshot | null>(null);

  useEffect(() => {
    // Initial fetch
    invoke<RawUsageSnapshot>('get_usage').then((raw) => setClaude(parseUsageSnapshot(raw))).catch(console.error);
    invoke<RawUsageSnapshot>('get_cursor').then((raw) => setCursor(parseUsageSnapshot(raw))).catch(console.error);
    invoke<RawUsageSnapshot>('get_codex').then((raw) => setCodex(parseUsageSnapshot(raw))).catch(console.error);
    invoke<RawUsageSnapshot>('get_antigravity').then((raw) => setAntigravity(parseUsageSnapshot(raw))).catch(console.error);

    // Setup listeners
    const unlistenUsage = listen<RawUsageSnapshot>('usage', (event) => {
      setClaude(parseUsageSnapshot(event.payload));
    });
    const unlistenCursor = listen<RawUsageSnapshot>('cursor', (event) => {
      setCursor(parseUsageSnapshot(event.payload));
    });
    const unlistenCodex = listen<RawUsageSnapshot>('codex', (event) => {
      setCodex(parseUsageSnapshot(event.payload));
    });
    const unlistenAntigravity = listen<RawUsageSnapshot>('antigravity', (event) => {
      setAntigravity(parseUsageSnapshot(event.payload));
    });

    return () => {
      unlistenUsage.then((f) => f());
      unlistenCursor.then((f) => f());
      unlistenCodex.then((f) => f());
      unlistenAntigravity.then((f) => f());
    };
  }, []);

  const refreshAll = () => {
    invoke('refresh_usage').catch(console.error);
  };

  // Filter out absent or none providers so the UI doesn't render them
  const activeProviders: Array<{ id: string; snapshot: UsageSnapshot }> = [];
  
  if (claude && claude.status.kind !== 'absent' && claude.status.kind !== 'none') {
    activeProviders.push({ id: 'claude', snapshot: claude });
  }
  if (cursor && cursor.status.kind !== 'absent' && cursor.status.kind !== 'none') {
    activeProviders.push({ id: 'cursor', snapshot: cursor });
  }
  if (codex && codex.status.kind !== 'absent' && codex.status.kind !== 'none') {
    activeProviders.push({ id: 'codex', snapshot: codex });
  }
  if (antigravity && antigravity.status.kind !== 'absent' && antigravity.status.kind !== 'none') {
    activeProviders.push({ id: 'antigravity', snapshot: antigravity });
  }

  return {
    providers: activeProviders,
    refreshAll,
  };
}
