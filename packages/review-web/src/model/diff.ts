export type Labels = Record<string, string>;

export interface DiffSpan {
  text: string;
  style: string;
  token: string;
  tooltip: string | null;
}

export interface DiffCell {
  oldLine: number | string | null;
  newLine: number | null;
  style: string;
  spans: DiffSpan[];
  noEofNewline: boolean;
  anchor: { side: "old" | "new"; line: number; group: number } | null;
}

export type DiffAnchor = NonNullable<DiffCell["anchor"]>;

export interface DiffView {
  language: string;
  added: number;
  removed: number;
  whitespaceOnlyHidden: boolean;
  rows: { cells: DiffCell[]; fullWidth: boolean }[];
}
