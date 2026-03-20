/**
 * useChartData hook
 *
 * フィルター設定に基づいてグラフデータを生成するカスタムフック
 */

import { useMemo } from "react";
import type { BenchmarkDataMap, ChartFilters, ChartState, MetricKey } from "../types";
import { processInstantPattern, processScalingPattern } from "../utils/chartData";

/**
 * メトリックの表示ラベルマップ
 */
const METRIC_LABELS: Map<MetricKey, string> = new Map([
  ["cycles", "CPU Cycles"],
  ["timeNs", "Time (ns)"],
  ["allocCount", "Allocations (Count)"],
  ["allocBytes", "Allocated Memory (Bytes)"],
]);

/**
 * グラフの空データ状態
 */
const EMPTY_CHART_STATE: ChartState = {
  chartData: [],
  lines: [],
  chartTitle: "",
  chartDesc: "",
  yAxisLabel: "",
  xAxisKey: "",
  xAxisLabel: "",
};

/**
 * フィルター設定からグラフ表示用のデータを生成するカスタムフック
 * @param benchmarkData - ベンチマークデータ全体
 * @param filters - フィルター設定
 * @param isInstant - Instantパターンかどうか
 * @returns グラフ表示用ステート
 */
export function useChartData(
  benchmarkData: BenchmarkDataMap,
  filters: ChartFilters,
  isInstant: boolean
): ChartState {
  return useMemo(() => {
    const { selectedFunc, selectedPattern, selectedMetric, historyCount } = filters;

    if (!selectedFunc || !selectedPattern) {
      return EMPTY_CHART_STATE;
    }

    const funcData = benchmarkData[selectedFunc];
    if (!funcData) {
      return EMPTY_CHART_STATE;
    }

    if (isInstant) {
      return processInstantPattern(funcData, selectedPattern, selectedMetric, METRIC_LABELS);
    }
    return processScalingPattern(
      funcData,
      selectedPattern,
      selectedMetric,
      historyCount,
      METRIC_LABELS
    );
  }, [benchmarkData, filters, isInstant]);
}

/**
 * メトリック表示ラベルを取得するヘルパー関数
 * @param metric - メトリックキー
 * @returns 表示ラベル
 */
export function getMetricLabel(metric: MetricKey): string {
  return METRIC_LABELS.get(metric) || metric;
}
