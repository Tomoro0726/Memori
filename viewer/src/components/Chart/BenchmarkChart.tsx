/**
 * BenchmarkChart component
 *
 * グラフ表示コンポーネント
 * 複数の子コンポーネント（グラフ、エクスポート、コントロール）をコンポーズして、
 * 統合されたグラフ表示インターフェースを提供
 */

import type React from "react";
import { useEffect, useRef, useState } from "react";
import type { LineInfo } from "../../types";
import styles from "./BenchmarkChart.module.css";
import { ChartCanvas } from "./ChartCanvas";
import { ChartControls } from "./ChartControls";
import { ChartExport } from "./ChartExport";

const DEFAULT_ALGO_COLORS = [
  "#2563eb",
  "#f97316",
  "#16a34a",
  "#dc2626",
  "#7c3aed",
  "#0ea5e9",
  "#d97706",
  "#059669",
  "#db2777",
  "#4f46e5",
];

const getDefaultAlgoColor = (index: number): string => {
  if (index < DEFAULT_ALGO_COLORS.length) {
    return DEFAULT_ALGO_COLORS[index];
  }

  // Fallback for many lines: maintain contrast with a hue wheel.
  return `hsl(${(index * 137.5) % 360}, 68%, 45%)`;
};

interface BenchmarkChartProps {
  /** グラフのタイトル */
  title: string;
  /** グラフの説明文 */
  description?: string;
  /** グラフ表示データ */
  data: Record<string, unknown>[];
  /** グラフに描画するラインリスト */
  lines: LineInfo[];
  /** Y軸のラベル */
  yAxisLabel: string;
  /** X軸のキー（データキー） */
  xAxisKey: string;
  /** X軸のラベル */
  xAxisLabel: string;
}

/**
 * ベンチマークデータをグラフ表示するコンポーネント
 * グラフのレンダリング、エクスポート機能、インタラクティブなコントロールを提供
 */
export const BenchmarkChart: React.FC<BenchmarkChartProps> = ({
  title,
  description,
  data,
  lines,
  yAxisLabel,
  xAxisKey,
  xAxisLabel,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(400);

  // アルゴリズム名のユニークなリスト
  const algoNames = Array.from(new Set(lines.map((l) => l.algoName)));

  // アルゴリズムの色マップ（自動生成）
  const [algoColors, setAlgoColors] = useState<Record<string, string>>({});

  // 初期化時に色を生成
  useEffect(() => {
    setAlgoColors((prev) => {
      const newColors = { ...prev };
      for (const [i, name] of algoNames.entries()) {
        if (!newColors[name]) {
          newColors[name] = getDefaultAlgoColor(i);
        }
      }
      return newColors;
    });
  }, [algoNames]);

  const handleAlgoColorChange = (algoName: string, color: string) => {
    setAlgoColors((prev) => ({ ...prev, [algoName]: color }));
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{title}</h3>
          {description && <p className={styles.description}>{description}</p>}
        </div>
        <ChartExport
          title={title}
          data={data}
          lines={lines}
          xAxisKey={xAxisKey}
          chartRef={chartRef}
        />
      </div>

      <ChartCanvas
        data={data}
        lines={lines}
        yAxisLabel={yAxisLabel}
        xAxisKey={xAxisKey}
        xAxisLabel={xAxisLabel}
        height={height}
        chartRef={chartRef}
        algoColors={algoColors}
      />

      <ChartControls
        height={height}
        onHeightChange={setHeight}
        algoNames={algoNames}
        algoColors={algoColors}
        onColorChange={handleAlgoColorChange}
      />
    </div>
  );
};
