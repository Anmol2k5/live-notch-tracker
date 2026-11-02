import { useState } from "react";
import { Icon } from "./Icon";

const quotes = [
  {
    text: "Stay hungry. Stay foolish.",
    author: "Steve Jobs",
    context: "For the days you need a little nudge.",
  },
  {
    text: "It always seems impossible until it’s done.",
    author: "Nelson Mandela",
    context: "One small step is still a step.",
  },
  {
    text: "Well done is better than well said.",
    author: "Benjamin Franklin",
    context: "Make something you’re proud of.",
  },
  {
    text: "Nothing in life is to be feared, it is only to be understood.",
    author: "Marie Curie",
    context: "Stay curious. Keep going.",
  },
];
export function QuoteView() {
  const [index, setIndex] = useState(0);
  const quote = quotes[index];
  return (
    <section className="widget quote-widget" aria-labelledby="quote-title">
      <div className="widget-heading">
        <h2 id="quote-title">
          <Icon name="quote" /> A moment of perspective
        </h2>
      </div>
      <div className="quote-body" aria-live="polite">
        <span className="quote-decoration" aria-hidden="true">
          “
        </span>
        <blockquote key={index}>{quote.text}</blockquote>
        <p>{quote.context}</p>
      </div>
      <div className="quote-footer">
        <cite>{quote.author}</cite>
        <button
          className="text-button"
          aria-label="Next quote"
          onClick={() => setIndex((i) => (i + 1) % quotes.length)}
        >
          Next <Icon name="arrow" size={15} />
        </button>
      </div>
    </section>
  );
}
