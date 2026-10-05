import { ProviderId, PROVIDER_METADATA } from '../model/providerTypes';

interface ProviderMarkProps {
  id: ProviderId;
  size?: number;
  className?: string;
}

export function ProviderMark({ id, size = 18, className = '' }: ProviderMarkProps) {
  const meta = PROVIDER_METADATA[id] ?? {
    id,
    displayName: id,
    shortCode: id.slice(0, 2).toUpperCase(),
    accentColor: '#FFFFFF',
  };

  switch (id) {
    case 'claude':
      // Anthropic spark / starburst mark
      return (
        <svg
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={className}
          aria-label={meta.displayName}
        >
          {/* 8-point asterisk / sunburst */}
          <line x1="12" y1="2" x2="12" y2="22" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
          <line x1="4.93" y1="19.07" x2="19.07" y2="4.93" />
          <circle cx="12" cy="12" r="2.5" fill="currentColor" />
        </svg>
      );

    case 'cursor':
      // Cursor isometric cube mark
      return (
        <svg
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.0"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={className}
          aria-label={meta.displayName}
        >
          <path d="M12 2.5 L20.5 7.5 L20.5 16.5 L12 21.5 L3.5 16.5 L3.5 7.5 Z" />
          <line x1="12" y1="2.5" x2="12" y2="21.5" />
          <line x1="12" y1="12" x2="20.5" y2="7.5" />
          <line x1="12" y1="12" x2="3.5" y2="7.5" />
        </svg>
      );

    case 'codex':
      // Codex / OpenAI spiral knot mark
      return (
        <svg
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.1"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={className}
          aria-label={meta.displayName}
        >
          <path d="M12 3a9 9 0 0 1 9 9c0 3.5-2 6.5-5 8" />
          <path d="M12 3a9 9 0 0 0-9 9c0 3.5 2 6.5 5 8" />
          <path d="M12 7.5a4.5 4.5 0 0 1 4.5 4.5c0 1.8-1 3.3-2.5 4" />
          <path d="M12 7.5a4.5 4.5 0 0 0-4.5 4.5c0 1.8 1 3.3 2.5 4" />
          <circle cx="12" cy="12" r="1.5" fill="currentColor" />
        </svg>
      );

    case 'antigravity':
      // Antigravity arched spark / portal mark
      return (
        <svg
          width={size}
          height={size}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.1"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={className}
          aria-label={meta.displayName}
        >
          <path d="M12 2 C12 7.5 7.5 12 2 12 C7.5 12 12 16.5 12 22 C12 16.5 16.5 12 22 12 C16.5 12 12 7.5 12 2 Z" fill="currentColor" fillOpacity="0.25" />
          <circle cx="12" cy="12" r="3" fill="currentColor" />
        </svg>
      );

    default:
      return (
        <span
          style={{
            fontSize: size * 0.75,
            fontWeight: 700,
            letterSpacing: '-0.02em',
            lineHeight: 1,
          }}
          aria-label={meta.displayName}
        >
          {meta.shortCode}
        </span>
      );
  }
}
