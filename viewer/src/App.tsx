/**
 * App.tsx
 *
 * ベンチマークビューアーのメインコンポーネント
 */

import { useEffect, useState } from "react";
import styles from "./App.module.css";
import { BenchmarkChart } from "./components/Chart";
import { FilterHeader } from "./components/Controls";
import { useChartData, useChartFilters, useIsInstantPattern } from "./hooks";
import type { BenchmarkDataMap, MetricKey } from "./types";
import { loadBenchmarkData } from "./utils/dataLoader";

/** メトリック表示ラベルマップ */
const METRICS: Array<{ key: MetricKey; label: string }> = [
  { key: "cycles", label: "CPU Cycles" },
  { key: "timeNs", label: "Time (ns)" },
  { key: "allocCount", label: "Allocations (Count)" },
  { key: "allocBytes", label: "Allocated Memory (Bytes)" },
];

/**
 * Appコンポーネント
 * ベンチマークデータをロードし、フィルターと グラフ表示を統合
 */
export default function App() {
  const [benchmarkData, setBenchmarkData] = useState<BenchmarkDataMap>({});
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setIsLoading(true);
      setLoadError(null);

      try {
        const data = await loadBenchmarkData();
        if (!cancelled) {
          setBenchmarkData(data);
        }
      } catch (err) {
        if (!cancelled) {
          console.error("Failed to load benchmark data:", err);
          setLoadError("Failed to load benchmark data.");
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    void load();

    return () => {
      cancelled = true;
    };
  }, []);

  const functions = Object.keys(benchmarkData);

  // フィルター状態の管理
  const { filters, setSelectedFunc, setSelectedPattern, setSelectedMetric, setHistoryCount } =
    useChartFilters(functions, benchmarkData);

  // Instant/Scaling パターンを判定
  const isInstant = useIsInstantPattern(
    benchmarkData,
    filters.selectedFunc,
    filters.selectedPattern
  );

  // グラフデータを生成
  const chartState = useChartData(benchmarkData, filters, isInstant);

  if (isLoading) {
    return <div className={styles.page}>Loading benchmark data...</div>;
  }

  if (loadError) {
    return <div className={styles.page}>{loadError}</div>;
  }

  if (functions.length === 0) {
    return <div className={styles.page}>No benchmark data found. Run Tenbin tests first!</div>;
  }

  return (
    <div className={styles.page}>
      <div className={styles.layout}>
        <FilterHeader
          functions={functions}
          selectedFunc={filters.selectedFunc}
          onSelectFunc={setSelectedFunc}
          benchmarkData={benchmarkData}
          selectedPattern={filters.selectedPattern}
          onSelectPattern={setSelectedPattern}
          metricLabels={METRICS}
          selectedMetric={filters.selectedMetric}
          onSelectMetric={setSelectedMetric}
          isInstant={isInstant}
          historyCount={filters.historyCount}
          onHistoryCountChange={setHistoryCount}
        />

        <main>
          {chartState.chartData.length > 0 ? (
            <BenchmarkChart
              title={chartState.chartTitle}
              description={chartState.chartDesc}
              data={chartState.chartData}
              lines={chartState.lines}
              yAxisLabel={chartState.yAxisLabel}
              xAxisKey={chartState.xAxisKey}
              xAxisLabel={chartState.xAxisLabel}
            />
          ) : (
            <div className={styles.noData}>
              Select a function and pattern to display the benchmark chart.
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
