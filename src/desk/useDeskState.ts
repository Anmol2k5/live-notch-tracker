import { useState } from "react";
import { DESK_KEY, DeskState, EMPTY_DESK, readDeskState } from "./model";

export function useDeskState() {
  const [initial] = useState(() => {
    try {
      return {
        state: readDeskState(localStorage.getItem(DESK_KEY)),
        error: "",
      };
    } catch (error) {
      return {
        state: EMPTY_DESK,
        error: String(error instanceof Error ? error.message : error),
      };
    }
  });
  const [state, setState] = useState(initial.state);
  const [error, setError] = useState(initial.error);
  const [blocked, setBlocked] = useState(Boolean(initial.error));
  function update(change: (s: DeskState) => DeskState): boolean {
    if (blocked) return false;
    try {
      const next = change(state);
      // Persist before updating the screen; failures must never look like a saved task.
      localStorage.setItem(DESK_KEY, JSON.stringify(next));
      setState(next);
      setError("");
      return true;
    } catch (error) {
      setError(
        error instanceof Error
          ? error.message
          : "Could not save your tasks. Check available storage.",
      );
      return false;
    }
  }
  function reset() {
    try {
      localStorage.setItem(DESK_KEY, JSON.stringify(EMPTY_DESK));
      setState(EMPTY_DESK);
      setError("");
      setBlocked(false);
    } catch {
      setError("Local storage is unavailable. Your tasks could not be saved.");
    }
  }
  return { state, update, error, blocked, reset };
}
