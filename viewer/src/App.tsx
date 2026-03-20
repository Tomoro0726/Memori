/**
 * App.tsx
 *
 * ベンチマークビューアーのメインコンポーネント
 */

import styles from "./App.module.css";
import { BenchmarkChart } from "./components/Chart";
import { FilterHeader } from "./components/Controls";
import { useChartData, useChartFilters, useIsInstantPattern } from "./hooks";
import type { MetricKey } from "./types";
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
  const benchmarkData = loadBenchmarkData();
  const functions = Object.keys(benchmarkData);

  // フィルター状態の管理
  const {
    filters,
    setSelectedFunc,
    setSelectedPattern,
    setSelectedMetric,
    setHistoryCount,
  } = useChartFilters(functions, benchmarkData);

  // Instant/Scaling パターンを判定
  const isInstant = useIsInstantPattern(
    benchmarkData,
    filters.selectedFunc,
    filters.selectedPattern,
  );

  // グラフデータを生成
  const chartState = useChartData(benchmarkData, filters, isInstant);

  if (functions.length === 0) {
    return (
      <div className={styles.page}>
        No benchmark data found. Run Tenbin tests first!
      </div>
    );
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
