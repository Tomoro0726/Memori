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
    selectedRuns: [0],
  });

  // 非同期ロード後に関数一覧が揃った場合、選択状態を補正する
  useEffect(() => {
    if (initialFunctions.length === 0) {
      return;
    }

    setFilters((prev) => {
      if (prev.selectedFunc && initialFunctions.includes(prev.selectedFunc)) {
        return prev;
      }

      return {
        ...prev,
        selectedFunc: initialFunctions[0],
        selectedPattern: "",
      };
    });
  }, [initialFunctions]);

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
    toggleSelectedRun: (runIndex: number) =>
      setFilters((prev) => {
        const exists = prev.selectedRuns.includes(runIndex);
        if (exists) {
          const next = prev.selectedRuns.filter((i) => i !== runIndex);
          return {
            ...prev,
            selectedRuns: next.length > 0 ? next : [0],
          };
        }

        return {
          ...prev,
          selectedRuns: [...prev.selectedRuns, runIndex].sort((a, b) => a - b),
        };
      }),
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
