/**
 * useChartFilters hook
 *
 * グラフコントローラーの状態管理を統一的に提供するカスタムフック
 */

import { useEffect, useState } from "react";
import type { BenchmarkDataMap, ChartFilters, MetricKey } from "../types";

/**
 * グラフの選択状態（関数、メトリック）を管理するカスタムフック
 * @param initialFunctions - 初期関数リスト
 * @param benchmarkData - ベンチマークデータ全体
 * @returns 状態管理オブジェクト
 */
export function useChartFilters(initialFunctions: string[]) {
  const [filters, setFilters] = useState<ChartFilters>({
    selectedFunc: initialFunctions[0] || "",
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
      };
    });
  }, [initialFunctions]);

  return {
    filters,
    setSelectedFunc: (func: string) => setFilters((prev) => ({ ...prev, selectedFunc: func })),
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
 * 関数が Scaling パターンを持っているか判定するフック
 * @param benchmarkData - ベンチマークデータ全体
 * @param selectedFunc - 選択関数
 * @returns Scalingパターンを1つでも持っていれば true
 */
export function useHasScalingPattern(
  benchmarkData: BenchmarkDataMap,
  selectedFunc: string
): boolean {
  const currentMeta = benchmarkData[selectedFunc]?.meta;
  if (!currentMeta) return false;
  return currentMeta.patterns.some((p) => p.patternType === "scaling");
}
