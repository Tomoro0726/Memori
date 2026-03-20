/**
 * ChartCanvas component
 *
 * Rechartsを使用したグラフ描画を担当する純粋なコンポーネント
 */

import type React from "react";
import { useMemo } from "react";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { LineInfo } from "../../types";
import styles from "./BenchmarkChart.module.css";

const FALLBACK_COLOR = "#2563eb";

function hexToRgb(hexColor: string): { r: number; g: number; b: number } {
  const hex = hexColor.replace("#", "").trim();
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) {
    return { r: 37, g: 99, b: 235 };
  }

  return {
    r: Number.parseInt(hex.slice(0, 2), 16),
    g: Number.parseInt(hex.slice(2, 4), 16),
    b: Number.parseInt(hex.slice(4, 6), 16),
  };
}

function toHex(value: number): string {
  return Math.max(0, Math.min(255, Math.round(value)))
    .toString(16)
    .padStart(2, "0");
}

function mixHexColors(baseColor: string, mixColor: string, ratio: number): string {
  const t = Math.max(0, Math.min(1, ratio));
  const base = hexToRgb(baseColor);
  const mix = hexToRgb(mixColor);

  const r = base.r * (1 - t) + mix.r * t;
  const g = base.g * (1 - t) + mix.g * t;
  const b = base.b * (1 - t) + mix.b * t;

  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

interface ChartCanvasProps {
  /** グラフ表示データ */
  data: Record<string, unknown>[];
  /** グラフに描画するラインリスト */
  lines: LineInfo[];
  /** Y軸のラベル */
  yAxisLabel: string;
  /** Xキー（データキー） */
  xAxisKey: string;
  /** X軸のラベル */
  xAxisLabel: string;
  /** グラフの高さ（px） */
  height: number;
  /** 参照node */
  chartRef: React.RefObject<HTMLDivElement | null>;
  /** アルゴリズムごとの色マップ */
  algoColors: Record<string, string>;
}

/**
 * Rechartsを使用したグラフキャンバスコンポーネント
 */
export const ChartCanvas: React.FC<ChartCanvasProps> = ({
  data,
  lines,
  yAxisLabel,
  xAxisKey,
  xAxisLabel,
  height,
  chartRef,
  algoColors,
}) => {
  // 最大実行インデックスを計算してopacityを決定
  const maxRunIndex = useMemo(() => Math.max(0, ...lines.map((l) => l.runIndex)), [lines]);

  const getSeriesColor = (line: LineInfo) => {
    const baseColor = algoColors[line.algoName] || FALLBACK_COLOR;
    if (maxRunIndex === 0) {
      return baseColor;
    }

    // Same algorithm family keeps hue; older runs are lightened.
    const ratio = (line.runIndex / maxRunIndex) * 0.5;
    return mixHexColors(baseColor, "#ffffff", ratio);
  };

  return (
    <div ref={chartRef} className={styles.chartArea} style={{ height: `${height}px` }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 10, right: 30, left: 60, bottom: 20 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />

          <XAxis
            dataKey={xAxisKey}
            label={{
              value: xAxisLabel,
              position: "insideBottomRight",
              offset: -10,
            }}
            padding={{ left: 20, right: 20 }}
          />

          <YAxis
            label={{
              value: yAxisLabel,
              angle: -90,
              position: "insideLeft",
              style: { textAnchor: "middle" },
              offset: -10,
            }}
          />

          <Tooltip
            formatter={(value: number | string | readonly (number | string)[] | undefined) => {
              if (typeof value === "number") return new Intl.NumberFormat().format(value);
              if (Array.isArray(value)) return value.join(", ");
              return value ?? "";
            }}
          />
          <Legend wrapperStyle={{ paddingTop: "20px" }} />

          {lines.map((line) => (
            <Line
              key={line.key}
              type="linear"
              dataKey={line.key}
              stroke={getSeriesColor(line)}
              strokeWidth={line.runIndex === 0 ? 3 : 2}
              strokeOpacity={1}
              dot={{ r: line.runIndex === 0 ? 4 : 2 }}
              activeDot={{ r: 6 }}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
