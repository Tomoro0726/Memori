/**
 * Benchmark data types
 *
 * ベンチマークデータのすべての型定義を統一的に管理するモジュール
 */

/** CPU測定値。CodSpeedおよびカスタムアロケーター統計 */
export interface Measurement {
  cycles: number;
  instructions: number | null;
  timeNs: number | null;
  allocCount: number;
  allocBytes: number;
  deallocCount: number;
  deallocBytes: number;
}

/** Measurement の任意のキーを取得する型ユーティリティ */
export type MetricKey = keyof Measurement;

/** 単一の入力サイズに対するベンチマーク結果 */
export interface BenchJsonEntry {
  input: number;
  measurement: Measurement;
}

/** ベンチマークパターン（Instant or Scaling）の結果 */
export interface BenchJsonReport {
  patternType: "instant" | "scaling";
  description?: string;
  results: Record<string, BenchJsonEntry[]>;
}

/** パターン定義メタデータ */
export interface PatternMetadata {
  name: string;
  description?: string;
  patternType: "instant" | "scaling";
}

/** 関数のメタデータ */
export interface FuncMetadata {
  name: string;
  description?: string;
  functions: string[];
  patterns: PatternMetadata[];
}

/** 実行履歴の1回分 */
export interface HistoryRun {
  fileName: string;
  data: Record<string, BenchJsonReport>;
}

/** 関数の完全なデータ（メタ + 履歴） */
export interface ParsedFunctionData {
  meta: FuncMetadata | null;
  history: HistoryRun[];
}

/** すべての関数のデータマップ */
export type BenchmarkDataMap = Record<string, ParsedFunctionData>;
