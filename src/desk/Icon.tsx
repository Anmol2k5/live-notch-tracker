export type IconName =
  | "grid"
  | "tasks"
  | "activity"
  | "github"
  | "quote"
  | "apps"
  | "plus"
  | "close"
  | "refresh"
  | "arrow"
  | "pin"
  | "compact"
  | "check"
  | "trash";
const paths: Record<IconName, string> = {
  grid: "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
  tasks: "M9 6h12 M9 12h12 M9 18h12 M3 6l1 1 2-2 M3 12l1 1 2-2 M3 18l1 1 2-2",
  activity: "M3 12h4l3-8 4 16 3-8h4",
  github:
    "M9 19c-5 1-5-3-7-3 M15 22v-4a3.5 3.5 0 0 0-1-2.8c3.3-.4 6.8-1.6 6.8-7.4a5.8 5.8 0 0 0-1.6-4 5.4 5.4 0 0 0-.1-4s-1.2-.4-4.1 1.6a14 14 0 0 0-7 0C5.1-.6 3.9-.2 3.9-.2a5.4 5.4 0 0 0-.1 4 5.8 5.8 0 0 0-1.6 4c0 5.8 3.5 7 6.8 7.4A3.5 3.5 0 0 0 8 18v4",
  quote:
    "M4 5h6v7H6c0 3 1 4 4 5v2c-5-1-7-4-7-8V6z M15 5h6v7h-4c0 3 1 4 4 5v2c-5-1-7-4-7-8V6z",
  apps: "M4 4h5v5H4z M15 4h5v5h-5z M4 15h5v5H4z M15 15h5v5h-5z",
  plus: "M12 5v14 M5 12h14",
  close: "M6 6l12 12 M18 6 6 18",
  refresh: "M20 7v5h-5 M20 12a8 8 0 1 0-2 5 M20 12 17 6",
  arrow: "M5 12h14 M14 7l5 5-5 5",
  pin: "m8 3 8 0-1 6 4 4H5l4-4z M12 13v8",
  compact: "M5 4h14v16H5z M15 4v16 M8 9h4 M8 13h4",
  check: "m5 12 4 4L19 6",
  trash: "M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7",
};
export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[name]} />
    </svg>
  );
}
