import { useState } from "react";
import {
  addProject,
  addTask,
  removeProject,
  removeTask,
  toggleTask,
} from "./model";
import { useDeskState } from "./useDeskState";
import { Icon } from "./Icon";

export function TasksView() {
  const { state, update, error, blocked, reset } = useDeskState();
  const [project, setProject] = useState(state.projects[0].id);
  const [addingProject, setAddingProject] = useState(false);
  const [name, setName] = useState("");
  const [title, setTitle] = useState("");
  const [deleting, setDeleting] = useState(false);
  const tasks = state.tasks.filter((t) => t.projectId === project);
  const done = tasks.filter((t) => t.done).length;
  return (
    <section className="widget tasks-widget" aria-labelledby="tasks-title">
      <div className="widget-heading">
        <h2 id="tasks-title">
          <Icon name="tasks" /> Your next little wins
        </h2>
        <span className="tag">
          {done}/{tasks.length} done
        </span>
      </div>
      <div className="project-strip">
        <div className="project-tabs" role="group" aria-label="Projects">
          {state.projects.map((p) => (
            <button
              key={p.id}
              className={
                p.id === project ? "project-tab selected" : "project-tab"
              }
              aria-pressed={p.id === project}
              onClick={() => {
                setProject(p.id);
                setDeleting(false);
              }}
            >
              {p.name}
            </button>
          ))}
        </div>
        <button
          className="icon-button"
          title="Add project"
          aria-label="Add project"
          onClick={() => setAddingProject(!addingProject)}
        >
          <Icon name="plus" size={16} />
        </button>
        {state.projects.length > 1 && (
          <button
            className="icon-button"
            aria-label="Delete current project"
            onClick={() => setDeleting(!deleting)}
          >
            <Icon name="trash" size={15} />
          </button>
        )}
      </div>
      {addingProject && (
        <form
          className="inline-form"
          onSubmit={(e) => {
            e.preventDefault();
            const id = crypto.randomUUID();
            if (update((s) => addProject(s, name, id))) {
              setProject(id);
              setName("");
              setAddingProject(false);
            }
          }}
        >
          <input
            autoFocus
            aria-label="Project name"
            placeholder="Project name"
            maxLength={40}
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
          <button className="small-button">Create</button>
        </form>
      )}
      {deleting && (
        <div className="confirm-row">
          Delete this project and its tasks?
          <button
            className="text-button danger"
            onClick={() => {
              if (update((s) => removeProject(s, project))) {
                setProject(state.projects.find((p) => p.id !== project)!.id);
                setDeleting(false);
              }
            }}
          >
            Delete
          </button>
          <button className="text-button" onClick={() => setDeleting(false)}>
            Cancel
          </button>
        </div>
      )}
      <div className="task-list">
        {!tasks.length && (
          <div className="task-empty">
            <span className="empty-orbit">
              <Icon name="check" size={22} />
            </span>
            <p>A little space to focus.</p>
            <span>Add the first task for this project.</span>
          </div>
        )}
        {tasks.map((task) => (
          <div
            className={`task-row ${task.done ? "completed" : ""}`}
            key={task.id}
          >
            <button
              className="task-toggle"
              role="checkbox"
              aria-checked={task.done}
              aria-label={`Complete ${task.title}`}
              onClick={() => update((s) => toggleTask(s, task.id))}
            >
              {task.done && <Icon name="check" size={13} />}
            </button>
            <span>{task.title}</span>
            <button
              className="icon-button task-delete"
              aria-label={`Delete ${task.title}`}
              onClick={() => update((s) => removeTask(s, task.id))}
            >
              <Icon name="close" size={14} />
            </button>
          </div>
        ))}
      </div>
      <form
        className="add-task"
        onSubmit={(e) => {
          e.preventDefault();
          if (update((s) => addTask(s, project, title, crypto.randomUUID())))
            setTitle("");
        }}
      >
        <Icon name="plus" size={16} />
        <input
          aria-label="New task"
          placeholder="What's next?"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          maxLength={200}
          required
          disabled={blocked}
        />
        <button className="small-button" disabled={blocked || !title.trim()}>
          Add task <span>↵</span>
        </button>
      </form>
      {error && (
        <div className="inline-error" role="alert">
          {error}
          {blocked && (
            <button className="text-button" onClick={reset}>
              Discard unreadable data & start fresh
            </button>
          )}
        </div>
      )}
      <div className="widget-foot">
        <span className={`status-dot ${error ? "muted" : ""}`} />
        {error ? "Local storage needs attention" : "Saved on this device"}
      </div>
    </section>
  );
}
