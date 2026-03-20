/**
 * Chart data processing utilities
 *
 * Instant パターンと Scaling パターンのベンチマークデータを
 * グラフ表示用の形式に変換するロジック
 */

import type {
  BenchJsonEntry,
  BenchmarkDataMap,
  ChartState,
  LineInfo,
  MetricKey,
} from "../types";

type ChartDataPoint = Record<string, unknown>;

/**
 * グラフラベルリストに新しいアルゴリズムを追加（重複チェック付き）
 * @param lineInfos - グラフラベル列
 * @param addedLineKeys - 既に追加されたキーセット
 * @param key - Recharts用のユニークキー
 * @param algoName - アルゴリズム名
 * @param runLabel - 実行ラベル
 * @param runIndex - 実行インデックス
 */
function addLineInfoIfNew(
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
  key: string,
  algoName: string,
  runLabel: string,
  runIndex: number,
): void {
  if (!addedLineKeys.has(key)) {
    addedLineKeys.add(key);
    lineInfos.push({ key, algoName, runLabel, runIndex });
  }
}

/**
 * Instant パターンのデータポイントに測定値を追加
 * @param algoName - アルゴリズム名
 * @param entries - ベンチマークエントリー
 * @param selectedMetric - 表示するメトリック
 * @param dataPoint - データポイント（参照経由で変更）
 */
function addInstantMetricToPoint(
  algoName: string,
  entries: BenchJsonEntry[],
  selectedMetric: MetricKey | string,
  dataPoint: ChartDataPoint,
): void {
  if (entries.length === 0) return;
  let value: unknown;
  if (
    [
      "cycles",
      "timeNs",
      "allocCount",
      "allocBytes",
      "deallocCount",
      "deallocBytes",
      "netBytes",
    ].includes(selectedMetric)
  ) {
    value = entries[0].measurement[selectedMetric as MetricKey];
  }
  if (value !== null && value !== undefined) {
    dataPoint[algoName] = value;
  }
}

/**
 * Scaling パターンのマップに測定値を追加
 * @param entries - ベンチマークエントリー
 * @param selectedMetric - 表示するメトリック
 * @param lineKey - グラフラベルキー
 * @param mergedMap - 入力サイズごとのデータマップ（参照経由で変更）
 */
function addScalingMetricsToMap(
  entries: BenchJsonEntry[],
  selectedMetric: MetricKey | "netBytes",
  lineKey: string,
  mergedMap: Map<number | string, ChartDataPoint>,
): void {
  for (const entry of entries) {
    let value: unknown;
    value = entry.measurement[selectedMetric as MetricKey];
    if (value === null || value === undefined) continue;

    if (!mergedMap.has(entry.input)) {
      mergedMap.set(entry.input, { input: entry.input });
    }
    const existing = mergedMap.get(entry.input);
    if (existing) {
      existing[lineKey] = value;
    }
  }
}

/**
 * 単一の実行履歴を Instant パターンで処理
 * @param run - 実行履歴データ
 * @param index - 実行インデックス
 * @param targetRunsLength - 対象実行数
 * @param selectedPattern - 選択パターン
 * @param selectedMetric - 選択メトリック
 * @param dataMap - データマップ（参照経由で変更）
 * @param lineInfos - グラフラベルリスト（参照経由で変更）
 * @param addedLineKeys - 追加済みキーセット（参照経由で変更）
 * @returns パターンの説明文（最新の場合のみ）
 */
function processInstantPatternRun(
  run: BenchmarkDataMap[string]["history"][number],
  index: number,
  targetRunsLength: number,
  selectedPattern: string,
  selectedMetric: MetricKey | string,
  dataMap: Map<number, ChartDataPoint>,
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
): string {
  const runNum = run.fileName.split("_")[0];
  const isLatest = index === targetRunsLength - 1;
  const runLabel = isLatest ? "Latest" : `Run-${runNum}`;

  const patternData = run.data[selectedPattern];
  if (!patternData) return "";

  const dataPoint: ChartDataPoint = { run: runLabel };

  for (const [algoName, entries] of Object.entries(patternData.results)) {
    addLineInfoIfNew(lineInfos, addedLineKeys, algoName, algoName, "", 0);
    addInstantMetricToPoint(algoName, entries, selectedMetric, dataPoint);
  }

  if (Object.keys(dataPoint).length > 1) {
    dataMap.set(index, dataPoint);
  }

  return isLatest ? patternData.description || "" : "";
}

/**
 * Instant パターンのすべての実行履歴を処理してグラフ状態を生成
 * @param funcData - 関数データ
 * @param selectedPattern - 選択パターン
 * @param selectedMetric - 選択メトリック
 * @param metricLabels - メトリック表示ラベルマップ
 * @returns グラフ表示用の状態
 */
export function processInstantPattern(
  funcData: BenchmarkDataMap[string],
  selectedPattern: string,
  selectedMetric: MetricKey,
  metricLabels: Map<MetricKey, string>,
): ChartState {
  const targetRuns = [...funcData.history].reverse();
  const trendMap = new Map<number, ChartDataPoint>();
  const lineInfos: LineInfo[] = [];
  const addedLineKeys = new Set<string>();
  let currentPatternDesc = "";

  for (const [index, run] of targetRuns.entries()) {
    const desc = processInstantPatternRun(
      run,
      index,
      targetRuns.length,
      selectedPattern,
      selectedMetric,
      trendMap,
      lineInfos,
      addedLineKeys,
    );
    if (desc) currentPatternDesc = desc;
  }

  const finalData = Array.from(trendMap.values());
  return {
    chartData: finalData,
    lines: lineInfos,
    chartTitle: `${selectedPattern} - Trend History`,
    chartDesc: currentPatternDesc || "CodSpeed style performance history.",
    yAxisLabel: metricLabels.get(selectedMetric) || selectedMetric,
    xAxisKey: "run",
    xAxisLabel: "History (Runs)",
  };
}

/**
 * 単一の実行履歴を Scaling パターンで処理
 * @param run - 実行履歴データ
 * @param index - 実行インデックス
 * @param selectedPattern - 選択パターン
 * @param selectedMetric - 選択メトリック
 * @param historyCount - 比較対象実行数
 * @param mergedMap - マージされたデータマップ（参照経由で変更）
 * @param lineInfos - グラフラベルリスト（参照経由で変更）
 * @param addedLineKeys - 追加済みキーセット（参照経由で変更）
 * @returns パターンの説明文（最初のみ）
 */
function processScalingPatternRun(
  run: BenchmarkDataMap[string]["history"][number],
  index: number,
  selectedPattern: string,
  selectedMetric: MetricKey,
  isSingleRun: boolean,
  mergedMap: Map<number | string, ChartDataPoint>,
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
): string {
  const runNum = run.fileName.replace(/\.json$/i, "").split("_")[0];
  const runLabel = index === 0 ? "Latest" : `Run-${runNum}`;
  const patternData = run.data[selectedPattern];

  if (!patternData) return "";

  for (const [algoName, entries] of Object.entries(patternData.results)) {
    const lineKey = isSingleRun ? algoName : `${algoName} (${runLabel})`;
    addLineInfoIfNew(
      lineInfos,
      addedLineKeys,
      lineKey,
      algoName,
      runLabel,
      index,
    );
    addScalingMetricsToMap(entries, selectedMetric, lineKey, mergedMap);
  }

  return index === 0 ? patternData.description || "" : "";
}

/**
 * Scaling パターンのすべての実行履歴を処理してグラフ状態を生成
 * @param funcData - 関数データ
 * @param selectedPattern - 選択パターン
 * @param selectedMetric - 選択メトリック
 * @param historyCount - 比較対象実行数
 * @param metricLabels - メトリック表示ラベルマップ
 * @returns グラフ表示用の状態
 */
export function processScalingPattern(
  funcData: BenchmarkDataMap[string],
  selectedPattern: string,
  selectedMetric: MetricKey,
  selectedRuns: number[],
  metricLabels: Map<MetricKey, string>,
): ChartState {
  const runIndices = selectedRuns.length > 0 ? selectedRuns : [0];
  const targetRuns = runIndices
    .map((runIndex) => ({ runIndex, run: funcData.history[runIndex] }))
    .filter(
      (
        entry,
      ): entry is {
        runIndex: number;
        run: BenchmarkDataMap[string]["history"][number];
      } => !!entry.run,
    );

  const mergedMap = new Map<number | string, ChartDataPoint>();
  const lineInfos: LineInfo[] = [];
  const addedLineKeys = new Set<string>();
  let currentPatternDesc = "";

  for (const entry of targetRuns) {
    const desc = processScalingPatternRun(
      entry.run,
      entry.runIndex,
      selectedPattern,
      selectedMetric,
      targetRuns.length === 1,
      mergedMap,
      lineInfos,
      addedLineKeys,
    );
    if (desc) currentPatternDesc = desc;
  }

  const finalData = Array.from(mergedMap.values()).sort((a, b) => {
    const valA = a.input;
    const valB = b.input;
    if (typeof valA === "number" && typeof valB === "number") {
      return valA - valB;
    }
    return String(valA).localeCompare(String(valB));
  });

  return {
    chartData: finalData,
    lines: lineInfos,
    chartTitle: `${selectedPattern} - ${metricLabels.get(selectedMetric)}`,
    chartDesc: currentPatternDesc,
    yAxisLabel: metricLabels.get(selectedMetric) || selectedMetric,
    xAxisKey: "input",
    xAxisLabel: "N (Input Size)",
  };
}
