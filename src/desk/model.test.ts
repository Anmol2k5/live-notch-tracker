import { describe, expect, it } from "vitest";
import {
  addProject,
  addTask,
  EMPTY_DESK,
  readDeskState,
  removeProject,
  removeTask,
  toggleTask,
} from "./model";

describe("local project tasks", () => {
  it("adds trimmed tasks to the chosen project and toggles without changing other tasks", () => {
    const state = addTask(
      EMPTY_DESK,
      "personal",
      "  Ship the Windows build  ",
      "t1",
    );
    const next = addTask(state, "personal", "Check DPI", "t2");
    expect(toggleTask(next, "t1").tasks).toEqual([
      {
        id: "t1",
        projectId: "personal",
        title: "Ship the Windows build",
        done: true,
      },
      { id: "t2", projectId: "personal", title: "Check DPI", done: false },
    ]);
    expect(state.tasks[0].done).toBe(false);
    expect(removeTask(next, "t1").tasks.map((t) => t.id)).toEqual(["t2"]);
  });
  it("rejects blank tasks, missing projects and duplicate project names", () => {
    expect(() => addTask(EMPTY_DESK, "personal", "  ", "t")).toThrow("task");
    expect(() => addTask(EMPTY_DESK, "missing", "Work", "t")).toThrow(
      "project",
    );
    expect(() => addProject(EMPTY_DESK, " personal ", "p2")).toThrow("exists");
  });
  it("removes only a deleted project’s tasks and keeps at least one project", () => {
    const state = addTask(
      addProject(EMPTY_DESK, "Work", "work"),
      "work",
      "Review",
      "t",
    );
    expect(removeProject(state, "work")).toEqual(EMPTY_DESK);
    expect(() => removeProject(EMPTY_DESK, "personal")).toThrow("one project");
  });
  it("round trips saved tasks and rejects corrupt or structurally invalid saved data", () => {
    const state = addTask(EMPTY_DESK, "personal", "Keep this task", "t");
    expect(readDeskState(JSON.stringify(state))).toEqual(state);
    expect(readDeskState(null)).toEqual(EMPTY_DESK);
    for (const saved of [
      "{",
      '{"version":7}',
      JSON.stringify({ ...state, tasks: [{ ...state.tasks[0], done: "yes" }] }),
      JSON.stringify({
        ...state,
        tasks: [{ ...state.tasks[0], projectId: "deleted" }],
      }),
    ]) {
      expect(() => readDeskState(saved)).toThrow("saved");
    }
  });
});
