import { invoke, isTauri } from "@tauri-apps/api/core";

export const desktop = isTauri();
// Serialize all window changes so a late notch placement cannot clip an expanded Desk.
let windowQueue = Promise.resolve();
export function windowJob(job: () => Promise<void>): Promise<void> {
  const pending = windowQueue.then(job);
  windowQueue = pending.catch(() => {});
  return pending;
}
export type AppId = "chrome" | "discord" | "spotify" | "obs";
export interface ContributionDay {
  date: string;
  count: number;
  level: number;
}
export interface GitHubCalendar {
  username: string;
  days: ContributionDay[];
}
export async function getCalendar(username: string): Promise<GitHubCalendar> {
  if (!desktop)
    throw new Error("Open the Windows app to load your GitHub activity.");
  return invoke("get_github_calendar", { username });
}
export async function launchApp(id: AppId): Promise<void> {
  if (!desktop)
    throw new Error("App shortcuts work in the Windows desktop app.");
  return invoke("launch_app", { id });
}
