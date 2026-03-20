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
 * フィルター設定からグラフ表示用のデータを生成するカスタムフック
 * @param benchmarkData - ベンチマークデータ全体
 * @param filters - フィルター設定
 * @returns グラフ表示用ステートの配列（パターンごと）
 */
export function useChartData(benchmarkData: BenchmarkDataMap, filters: ChartFilters): ChartState[] {
  return useMemo(() => {
    const { selectedFunc, selectedMetric, selectedRuns } = filters;

    if (!selectedFunc) {
      return [];
    }

    const funcData = benchmarkData[selectedFunc];
    if (!funcData || !funcData.meta) {
      return [];
    }

    const charts: ChartState[] = [];

    // Funcに含まれるすべてのパターンを処理してグラフデータの配列を作る
    for (const pattern of funcData.meta.patterns) {
      const isInstant = pattern.patternType === "instant";
      if (isInstant) {
        charts.push(processInstantPattern(funcData, pattern.name, selectedMetric, METRIC_LABELS));
      } else {
        charts.push(
          processScalingPattern(funcData, pattern.name, selectedMetric, selectedRuns, METRIC_LABELS)
        );
      }
    }

    return charts;
  }, [benchmarkData, filters]);
}

/**
 * メトリック表示ラベルを取得するヘルパー関数
 * @param metric - メトリックキー
 * @returns 表示ラベル
 */
export function getMetricLabel(metric: MetricKey): string {
  return METRIC_LABELS.get(metric) || metric;
}
