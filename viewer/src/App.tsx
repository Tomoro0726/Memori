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
import { loadBenchmarkData, loadBenchmarkDataFromFileList } from "./utils/dataLoader";

/** メトリック表示ラベルマップ */
const METRICS: Array<{ key: MetricKey; label: string }> = [
  { key: "cycles", label: "CPU Cycles" },
  { key: "timeNs", label: "Time (ns)" },
  { key: "allocCount", label: "Allocations (Count)" },
  { key: "allocBytes", label: "Allocated Memory (Bytes)" },
];

function getLoadFailureState(): {
  needsLocalFolderSelection: boolean;
  message: string;
} {
  const isFileProtocol = window.location.protocol === "file:";
  if (isFileProtocol) {
    return {
      needsLocalFolderSelection: true,
      message: "file:// では自動ロードできません。target/tenbin フォルダを選択してください。",
    };
  }

  return {
    needsLocalFolderSelection: false,
    message: "Failed to load benchmark data.",
  };
}

/**
 * Appコンポーネント
 * ベンチマークデータをロードし、フィルターと グラフ表示を統合
 */
export default function App() {
  const [benchmarkData, setBenchmarkData] = useState<BenchmarkDataMap>({});
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [needsLocalFolderSelection, setNeedsLocalFolderSelection] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const ifActive = (fn: () => void) => {
      if (!cancelled) {
        fn();
      }
    };

    const handleLoadError = (err: unknown) => {
      console.error("Failed to load benchmark data:", err);
      const failure = getLoadFailureState();
      setNeedsLocalFolderSelection(failure.needsLocalFolderSelection);
      setLoadError(failure.message);
    };

    const load = async () => {
      setIsLoading(true);
      setLoadError(null);

      try {
        const data = await loadBenchmarkData();
        ifActive(() => setBenchmarkData(data));
      } catch (err) {
        ifActive(() => handleLoadError(err));
      } finally {
        ifActive(() => setIsLoading(false));
      }
    };

    void load();

    return () => {
      cancelled = true;
    };
  }, []);

  const handleFolderSelection = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) {
      return;
    }

    setIsLoading(true);
    setLoadError(null);

    try {
      const data = await loadBenchmarkDataFromFileList(files);
      setBenchmarkData(data);
      setNeedsLocalFolderSelection(false);
    } catch (err) {
      console.error("Failed to load local benchmark data:", err);
      setLoadError("フォルダ内JSONの読み込みに失敗しました。target/tenbin を選択してください。");
    } finally {
      setIsLoading(false);
    }
  };

  const functions = Object.keys(benchmarkData);

  // フィルター状態の管理
  const { filters, setSelectedFunc, setSelectedPattern, setSelectedMetric, toggleSelectedRun } =
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
    if (needsLocalFolderSelection) {
      return (
        <div className={styles.page}>
          <div className={styles.localLoadPanel}>
            <h2 className={styles.localLoadTitle}>Local Folder Selection Required</h2>
            <p className={styles.localLoadText}>{loadError}</p>
            <p className={styles.localLoadText}>フォルダは target/tenbin を選択してください。</p>
            <input
              className={styles.localLoadInput}
              type="file"
              multiple
              // @ts-expect-error webkitdirectory is non-standard but supported by Chromium
              webkitdirectory=""
              onChange={handleFolderSelection}
            />
          </div>
        </div>
      );
    }

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
          selectedRuns={filters.selectedRuns}
          onToggleSelectedRun={toggleSelectedRun}
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
