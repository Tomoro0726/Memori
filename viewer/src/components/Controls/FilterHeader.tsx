/**
 * FilterHeader component
 *
 * ベンチマークデータのフィルター（関数、メトリック、履歴数）を作成するコンポーネント
 */

import type React from "react";
import styles from "../../App.module.css";
import type { BenchmarkDataMap, MetricKey } from "../../types";

interface FilterHeaderProps {
  /** すべての関数リスト */
  functions: string[];
  /** 選択されている関数 */
  selectedFunc: string;
  /** 関数選択時のコールバック */
  onSelectFunc: (func: string) => void;
  /** ベンチマークデータ */
  benchmarkData: BenchmarkDataMap;
  /** メトリックのラベルマップ */
  metricLabels: Array<{ key: MetricKey; label: string }>;
  /** 選択されているメトリック */
  selectedMetric: MetricKey;
  /** メトリック選択時のコールバック */
  onSelectMetric: (metric: MetricKey) => void;
  /** Scalingパターンが存在するかどうか（Compare History表示用） */
  hasScaling: boolean;
  /** 選択されている実行履歴インデックス群 */
  selectedRuns: number[];
  /** 実行履歴トグル時のコールバック */
  onToggleSelectedRun: (runIndex: number) => void;
}

/**
 * ページトップのフィルターヘッダーコンポーネント
 * グラフ表示のためのフィルター条件を選択するインターフェース を提供
 */
export const FilterHeader: React.FC<FilterHeaderProps> = ({
  functions,
  selectedFunc,
  onSelectFunc,
  benchmarkData,
  metricLabels,
  selectedMetric,
  onSelectMetric,
  hasScaling,
  selectedRuns,
  onToggleSelectedRun,
}) => {
  const historyRuns = benchmarkData[selectedFunc]?.history || [];

  return (
    <header className={styles.header}>
      <div>
        <h1 className={styles.pageTitle}>memori Viewer</h1>
        <p className={styles.pageSubtitle}>Rust Performance Benchmarks</p>
      </div>
      <div className={styles.spacer} />

      <div className={styles.filters}>
        {/* Target Function */}
        <div className={styles.filterGroup}>
          <label className={styles.filterLabel} htmlFor="target-func">
            Target Function
          </label>
          <select
            id="target-func"
            value={selectedFunc}
            onChange={(e) => onSelectFunc(e.target.value)}
            className={styles.selectBox}
          >
            {functions.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </div>

        {/* Metric to Graph */}
        <div className={styles.filterGroup}>
          <label className={styles.filterLabel} htmlFor="metric-graph">
            Metric to Graph
          </label>
          <select
            id="metric-graph"
            value={selectedMetric}
            onChange={(e) => onSelectMetric(e.target.value as MetricKey)}
            className={styles.selectBox}
          >
            {metricLabels.map((m) => (
              <option key={m.key} value={m.key}>
                {m.label}
              </option>
            ))}
          </select>
        </div>

        {/* Compare History (Scalingが含まれるときだけ表示) */}
        {hasScaling && (
          <div className={styles.filterGroup}>
            <span className={styles.filterLabel}>Compare History</span>
            <details className={styles.historyDisclosure}>
              <summary className={styles.historySummary}>
                {selectedRuns.length} runs selected
              </summary>

              <div className={styles.historyChecks}>
                {historyRuns.map((run, index) => {
                  const runNum = run.fileName.replace(/\.json$/i, "").split("_")[0];
                  const runLabel = index === 0 ? "Latest" : `Run-${runNum}`;

                  return (
                    <label key={run.fileName} className={styles.historyCheckItem}>
                      <input
                        type="checkbox"
                        checked={selectedRuns.includes(index)}
                        onChange={() => onToggleSelectedRun(index)}
                      />
                      {runLabel}
                    </label>
                  );
                })}
              </div>
            </details>
          </div>
        )}
      </div>
    </header>
  );
};
