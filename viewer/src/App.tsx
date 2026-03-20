/**
 * App.tsx
 *
 * ベンチマークビューアーのメインコンポーネント
 */

import { useEffect, useState } from "react";
import styles from "./App.module.css";
import { BenchmarkChart } from "./components/Chart";
import { FilterHeader } from "./components/Controls";
import { useChartData, useChartFilters, useHasScalingPattern } from "./hooks";
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
      message: "file:// では自動ロードできません。target/memori フォルダを選択してください。",
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
      setLoadError("フォルダ内JSONの読み込みに失敗しました。target/memori を選択してください。");
    } finally {
      setIsLoading(false);
    }
  };

  const functions = Object.keys(benchmarkData);

  // フィルター状態の管理
  const { filters, setSelectedFunc, setSelectedMetric, toggleSelectedRun } =
    useChartFilters(functions);

  // 関数にScalingパターンが含まれているかを判定（Compare History の表示制御用）
  const hasScaling = useHasScalingPattern(benchmarkData, filters.selectedFunc);

  // グラフデータ（複数パターン）を生成
  const charts = useChartData(benchmarkData, filters);

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
            <p className={styles.localLoadText}>フォルダは target/memori を選択してください。</p>
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
    return <div className={styles.page}>No benchmark data found. Run memori tests first!</div>;
  }

  return (
    <div className={styles.page}>
      <div className={styles.layout}>
        <FilterHeader
          functions={functions}
          selectedFunc={filters.selectedFunc}
          onSelectFunc={setSelectedFunc}
          benchmarkData={benchmarkData}
          metricLabels={METRICS}
          selectedMetric={filters.selectedMetric}
          onSelectMetric={setSelectedMetric}
          hasScaling={hasScaling}
          selectedRuns={filters.selectedRuns}
          onToggleSelectedRun={toggleSelectedRun}
        />

        <main>
          {charts.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: "2rem" }}>
              {charts.map((chart, idx) => (
                <BenchmarkChart
                  key={chart.chartTitle || idx}
                  title={chart.chartTitle}
                  description={chart.chartDesc}
                  data={chart.chartData}
                  lines={chart.lines}
                  yAxisLabel={chart.yAxisLabel}
                  xAxisKey={chart.xAxisKey}
                  xAxisLabel={chart.xAxisLabel}
                />
              ))}
            </div>
          ) : (
            <div>Select a function to display the benchmark charts.</div>
          )}
        </main>
      </div>
    </div>
  );
}
