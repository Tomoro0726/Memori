/**
 * FilterHeader component
 *
 * ベンチマークデータのフィルター（関数、パターン、メトリック、履歴数）を作成するコンポーネント
 */

import type React from "react";
import type { BenchmarkDataMap, MetricKey } from "../../types";
import styles from "../../App.module.css";

interface FilterHeaderProps {
  /** すべての関数リスト */
  functions: string[];
  /** 選択されている関数 */
  selectedFunc: string;
  /** 関数選択時のコールバック */
  onSelectFunc: (func: string) => void;
  /** ベンチマークデータ */
  benchmarkData: BenchmarkDataMap;
  /** 選択されているパターン */
  selectedPattern: string;
  /** パターン選択時のコールバック */
  onSelectPattern: (pattern: string) => void;
  /** メトリックのラベルマップ */
  metricLabels: Array<{ key: MetricKey; label: string }>;
  /** 選択されているメトリック */
  selectedMetric: MetricKey;
  /** メトリック選択時のコールバック */
  onSelectMetric: (metric: MetricKey) => void;
  /** Instantパターンかどうか */
  isInstant: boolean;
  /** 選択されている履歴数 */
  historyCount: number;
  /** 履歴数変更時のコールバック */
  onHistoryCountChange: (count: number) => void;
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
  selectedPattern,
  onSelectPattern,
  metricLabels,
  selectedMetric,
  onSelectMetric,
  isInstant,
  historyCount,
  onHistoryCountChange,
}) => {
  const currentMeta = benchmarkData[selectedFunc]?.meta;

  return (
    <header className={styles.header}>
      <div>
        <h1 className={styles.pageTitle}>Tenbin Viewer</h1>
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

        {/* Benchmark Pattern */}
        <div className={styles.filterGroup}>
          <label className={styles.filterLabel} htmlFor="bench-pattern">
            Benchmark Pattern
          </label>
          <select
            id="bench-pattern"
            value={selectedPattern}
            onChange={(e) => onSelectPattern(e.target.value)}
            className={styles.selectBox}
          >
            {currentMeta?.patterns.map((p) => (
              <option key={p.name} value={p.name}>
                {p.name}
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

        {/* Compare History (Scalingのときだけ表示) */}
        {!isInstant && (
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel} htmlFor="compare-history">
              Compare History
            </label>
            <select
              id="compare-history"
              value={historyCount}
              onChange={(e) => onHistoryCountChange(Number(e.target.value))}
              className={styles.selectBox}
            >
              <option value={1}>Latest only</option>
              <option value={2}>Last 2 runs</option>
              <option value={3}>Last 3 runs</option>
              <option value={5}>Last 5 runs</option>
            </select>
          </div>
        )}
      </div>
    </header>
  );
};
