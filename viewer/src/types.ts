export interface Measurement {
  cycles: number;
  instructions: number | null;
  timeNs: number | null;
  allocCount: number;
  allocBytes: number;
  deallocCount: number;
  deallocBytes: number;
}

export type MetricKey = keyof Measurement;

export interface BenchJsonEntry {
  input: number;
  measurement: Measurement;
}

export interface BenchJsonReport {
  patternType: "instant" | "scaling";
  description?: string;
  results: Record<string, BenchJsonEntry[]>;
}

export interface PatternMetadata {
  name: string;
  description?: string;
  patternType: "instant" | "scaling";
}

export interface FuncMetadata {
  name: string;
  description?: string;
  functions: string[];
  patterns: PatternMetadata[];
}

export interface HistoryRun {
  fileName: string;
  data: Record<string, BenchJsonReport>;
}

export interface ParsedFunctionData {
  meta: FuncMetadata | null;
  history: HistoryRun[];
}

export type BenchmarkDataMap = Record<string, ParsedFunctionData>;

export interface LineInfo {
  key: string; // Recharts用のユニークキー (例: "HashSet (Latest)")
  algoName: string; // アルゴリズム名 (例: "HashSet")
  runLabel: string; // 実行ラベル (例: "Latest", "Run-001")
  runIndex: number; // 最新からのインデックス (0 = 最新, 1 = 1回前...)
}
