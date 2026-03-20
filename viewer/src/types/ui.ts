/**
 * UI-related types
 *
 * グラフ表示に必要な UI 特有の型定義
 */

import type { MetricKey } from "./benchmark";

/** グラフ内に表示するデータシリーズに関する情報 */
export interface LineInfo {
  /** Recharts用のユニークキー (例: "HashSet (Latest)") */
  key: string;
  /** アルゴリズム名 (例: "HashSet") */
  algoName: string;
  /** 実行ラベル (例: "Latest", "Run-001") */
  runLabel: string;
  /** 最新からのインデックス (0 = 最新, 1 = 1回前...) */
  runIndex: number;
}

/** グラフ表示に必要なすべてのデータ */
export interface ChartState {
  chartData: Record<string, unknown>[];
  lines: LineInfo[];
  chartTitle: string;
  chartDesc: string;
  yAxisLabel: string;
  xAxisKey: string;
  xAxisLabel: string;
}

/** グラフコントローラー内の選択状態 */
export interface ChartFilters {
  selectedFunc: string;
  selectedPattern: string;
  selectedMetric: MetricKey;
  selectedRuns: number[];
}
