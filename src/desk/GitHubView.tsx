import { useEffect, useRef, useState } from "react";
import { desktop, getCalendar, GitHubCalendar } from "./native";
import { Icon } from "./Icon";

const KEY = "codenotch.github.username";
function savedUsername() {
  try {
    return localStorage.getItem(KEY) || "";
  } catch {
    return "";
  }
}
export function GitHubView() {
  const [username, setUsername] = useState(savedUsername);
  const [calendar, setCalendar] = useState<GitHubCalendar | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const request = useRef(0);
  async function load(name: string) {
    const ticket = ++request.current;
    setCalendar(null);
    setError("");
    name = name.trim();
    if (!/^[a-z\d](?:[a-z\d-]{0,37}[a-z\d])?$/i.test(name)) {
      setError("Enter a valid GitHub username.");
      return;
    }
    setLoading(true);
    try {
      const result = await getCalendar(name);
      if (ticket !== request.current) return;
      setCalendar(result);
      try {
        localStorage.setItem(KEY, name);
      } catch {
        setError("Activity loaded, but your username could not be saved.");
      }
    } catch (e) {
      if (ticket === request.current)
        setError(String(e instanceof Error ? e.message : e));
    } finally {
      if (ticket === request.current) setLoading(false);
    }
  }
  useEffect(() => {
    if (desktop && username) void load(username);
    return () => {
      request.current++;
    };
    // Only restore the saved account on mount; typing never issues requests.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  const days = calendar?.days.slice(-126) ?? [];
  const offset = days.length
    ? new Date(`${days[0].date}T00:00:00Z`).getUTCDay()
    : 0;
  const total = days.reduce((n, d) => n + d.count, 0);
  return (
    <section className="widget github-widget" aria-labelledby="github-title">
      <div className="widget-heading">
        <h2 id="github-title">
          <Icon name="github" /> A little, every day
        </h2>
        {calendar && (
          <button
            className="icon-button"
            aria-label="Refresh GitHub activity"
            disabled={loading}
            onClick={() => void load(calendar.username)}
          >
            <Icon name="refresh" size={15} />
          </button>
        )}
      </div>
      <div className="github-identity">
        <span className="github-avatar">
          <Icon name="github" size={20} />
        </span>
        <div>
          <strong>
            {calendar ? `@${calendar.username}` : "Your GitHub activity"}
          </strong>
          <span>
            {calendar
              ? `${total.toLocaleString()} contributions · last 18 weeks`
              : "Good things add up."}
          </span>
        </div>
      </div>
      <div
        className={`contribution-grid ${calendar ? "" : "empty-grid"}`}
        aria-label={
          calendar
            ? `${total} public contributions in the last 18 weeks`
            : "Connect GitHub to see your contributions"
        }
      >
        {calendar ? (
          <>
            {Array.from({ length: offset }, (_, i) => (
              <span key={`blank-${i}`} className="contribution-blank" />
            ))}
            {days.map((d) => (
              <span
                key={d.date}
                className={`contribution level-${d.level}`}
                title={`${d.count} contributions on ${d.date}`}
              />
            ))}
          </>
        ) : (
          Array.from({ length: 126 }, (_, i) => (
            <span className="contribution level-0" key={i} />
          ))
        )}
      </div>
      {calendar && (
        <div className="heatmap-legend">
          <span>Less</span>
          {[0, 1, 2, 3, 4].map((n) => (
            <i key={n} className={`contribution level-${n}`} />
          ))}
          <span>More</span>
        </div>
      )}
      <form
        className="github-form"
        onSubmit={(e) => {
          e.preventDefault();
          void load(username);
        }}
      >
        <span>@</span>
        <input
          aria-label="GitHub username"
          placeholder="GitHub username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          maxLength={39}
          required
        />
        <button
          className="icon-button"
          aria-label="Load GitHub activity"
          disabled={loading || !username.trim()}
        >
          {loading ? (
            <span className="loading-dots">…</span>
          ) : (
            <Icon name="arrow" size={17} />
          )}
        </button>
      </form>
      {error && (
        <p className="inline-error" role="alert">
          {error}
        </p>
      )}
      <div className="widget-foot">
        {loading
          ? "Loading public contributions…"
          : calendar
            ? "Public activity from github.com"
            : "Connect a public profile to get started"}
      </div>
    </section>
  );
}
