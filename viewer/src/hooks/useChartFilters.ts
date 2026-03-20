/**
 * useChartFilters hook
 *
 * グラフコントローラーの状態管理を統一的に提供するカスタムフック
 */

import { useEffect, useState } from "react";
import type { BenchmarkDataMap, ChartFilters, MetricKey } from "../types";

/**
 * グラフの選択状態（関数、パターン、メトリック）を管理するカスタムフック
 * @param initialFunctions - 初期関数リスト
 * @param benchmarkData - ベンチマークデータ全体
 * @returns 状態管理オブジェクト
 */
export function useChartFilters(initialFunctions: string[], benchmarkData: BenchmarkDataMap) {
  const [filters, setFilters] = useState<ChartFilters>({
    selectedFunc: initialFunctions[0] || "",
    selectedPattern: "",
    selectedMetric: "cycles",
    historyCount: 1,
  });

  // 関数が変更されたら、最初のパターンを自動選択
  useEffect(() => {
    if (filters.selectedFunc && benchmarkData[filters.selectedFunc]?.meta) {
      const patterns = benchmarkData[filters.selectedFunc].meta?.patterns;
      if (patterns && patterns.length > 0) {
        setFilters((prev) => ({ ...prev, selectedPattern: patterns[0].name }));
      }
    }
  }, [filters.selectedFunc, benchmarkData]);

  return {
    filters,
    setSelectedFunc: (func: string) => setFilters((prev) => ({ ...prev, selectedFunc: func })),
    setSelectedPattern: (pattern: string) =>
      setFilters((prev) => ({ ...prev, selectedPattern: pattern })),
    setSelectedMetric: (metric: MetricKey) =>
      setFilters((prev) => ({ ...prev, selectedMetric: metric })),
    setHistoryCount: (count: number) => setFilters((prev) => ({ ...prev, historyCount: count })),
  };
}

/**
 * パターンタイプ（Instant or Scaling）を判定するカスタムフック
 * @param benchmarkData - ベンチマークデータ全体
 * @param selectedFunc - 選択関数
 * @param selectedPattern - 選択パターン
 * @returns Instantパターンであれば true、Scaling であれば false
 */
export function useIsInstantPattern(
  benchmarkData: BenchmarkDataMap,
  selectedFunc: string,
  selectedPattern: string
): boolean {
  const currentMeta = benchmarkData[selectedFunc]?.meta;
  const currentPatternMeta = currentMeta?.patterns.find((p) => p.name === selectedPattern);
  return currentPatternMeta?.patternType === "instant";
}
