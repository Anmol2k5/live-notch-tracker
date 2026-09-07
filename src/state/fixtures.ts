export interface FixtureCell {
  id: string;
  initial: string;
  percent: number;
}

export const fixtures: FixtureCell[] = [
  { id: 'claude', initial: 'C', percent: 73 },
  { id: 'openai', initial: 'O', percent: 21 },
  { id: 'perplexity', initial: 'P', percent: 52 },
];
