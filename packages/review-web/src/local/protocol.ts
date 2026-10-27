import type { DiffOptions, ReviewAction } from "../model/review";

export type WorkerRequest =
  | { kind: "init"; databaseName: string }
  | { kind: "open"; bundle?: string }
  | { kind: "render"; session: number; file: number; options: DiffOptions }
  | { kind: "mutate"; session: number; file: number; options: DiffOptions; action: ReviewAction }
  | { kind: "restoreState"; session: number; file: number; options: DiffOptions }
  | { kind: "export"; session: number }
  | { kind: "importState"; session: number; file: number; options: DiffOptions; state: string };

export type WorkerResponse = { id: number; result: unknown } | { id: number; error: string };
