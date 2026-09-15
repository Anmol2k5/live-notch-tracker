export const palette = {
  notch: '#000000',
  notchHighlight: '#0d0d0d',   // subtle lift for gradient depth
  card: '#000000',
  ringTrack: '#1e1e1e',
  barTrack: '#2D2D2D',
  ample: '#34d399',            // emerald-400 — rich but not neon
  watch: '#facc15',            // amber-400 — warm gold
  critical: '#f87171',         // red-400 — urgent but not screaming
  textPrimary: 'rgba(255,255,255,0.95)',
  textSecondary: 'rgba(255,255,255,0.45)',
  textMuted: 'rgba(255,255,255,0.30)',
} satisfies Record<string, string>;

/** Per-band glow color at reduced opacity for the SVG blur filter */
export const glowColor: Record<string, string> = {
  ample: 'rgba(52,211,153,0.45)',
  watch: 'rgba(250,204,21,0.40)',
  critical: 'rgba(248,113,113,0.55)',
};
