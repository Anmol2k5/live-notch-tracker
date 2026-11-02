export interface Project {
  id: string;
  name: string;
}
export interface Task {
  id: string;
  projectId: string;
  title: string;
  done: boolean;
}
export interface DeskState {
  version: 1;
  projects: Project[];
  tasks: Task[];
}
export const DESK_KEY = "codenotch.desk.v1";
export const EMPTY_DESK: DeskState = {
  version: 1,
  projects: [{ id: "personal", name: "Personal" }],
  tasks: [],
};

export function readDeskState(raw: string | null): DeskState {
  if (raw === null) return EMPTY_DESK;
  try {
    const s = JSON.parse(raw);
    const text = (v: unknown, max: number): v is string =>
      typeof v === "string" && v.trim().length > 0 && v.length <= max;
    if (
      s.version !== 1 ||
      !Array.isArray(s.projects) ||
      !s.projects.length ||
      s.projects.length > 50 ||
      !Array.isArray(s.tasks) ||
      s.tasks.length > 500 ||
      !s.projects.every(
        (p: Project) => p && text(p.id, 100) && text(p.name, 40),
      ) ||
      new Set(s.projects.map((p: Project) => p.id)).size !==
        s.projects.length ||
      !s.tasks.every(
        (t: Task) =>
          t &&
          text(t.id, 100) &&
          text(t.title, 200) &&
          typeof t.done === "boolean" &&
          s.projects.some((p: Project) => p.id === t.projectId),
      ) ||
      new Set(s.tasks.map((t: Task) => t.id)).size !== s.tasks.length
    )
      throw new Error();
    return { version: 1, projects: s.projects, tasks: s.tasks };
  } catch {
    throw new Error(
      "Your saved tasks could not be read. They have been kept untouched.",
    );
  }
}

export function addTask(
  s: DeskState,
  projectId: string,
  title: string,
  id: string,
): DeskState {
  title = title.trim();
  if (!title || title.length > 200)
    throw new Error("Write a task between 1 and 200 characters.");
  if (!s.projects.some((p) => p.id === projectId))
    throw new Error("Choose a project first.");
  if (s.tasks.length >= 500)
    throw new Error("Your desk is full. Remove a task to add another.");
  return { ...s, tasks: [...s.tasks, { id, projectId, title, done: false }] };
}
export function toggleTask(s: DeskState, id: string): DeskState {
  return {
    ...s,
    tasks: s.tasks.map((t) => (t.id === id ? { ...t, done: !t.done } : t)),
  };
}
export function removeTask(s: DeskState, id: string): DeskState {
  return { ...s, tasks: s.tasks.filter((t) => t.id !== id) };
}
export function addProject(s: DeskState, name: string, id: string): DeskState {
  name = name.trim();
  if (!name || name.length > 40)
    throw new Error("Use a project name between 1 and 40 characters.");
  if (s.projects.some((p) => p.name.toLowerCase() === name.toLowerCase()))
    throw new Error("That project already exists.");
  if (s.projects.length >= 50)
    throw new Error("You can keep up to 50 projects.");
  return { ...s, projects: [...s.projects, { id, name }] };
}
export function removeProject(s: DeskState, id: string): DeskState {
  if (s.projects.length === 1)
    throw new Error("Keep at least one project on your desk.");
  return {
    ...s,
    projects: s.projects.filter((p) => p.id !== id),
    tasks: s.tasks.filter((t) => t.projectId !== id),
  };
}
