/**
 * ChartControls component
 *
 * グラフの表示設定（高さ、色）を制御するコンポーネント
 */

import { Settings } from "lucide-react";
import type React from "react";
import styles from "./BenchmarkChart.module.css";

interface ChartControlsProps {
  /** グラフの高さ（px） */
  height: number;
  /** 高さ変更時のコールバック */
  onHeightChange: (height: number) => void;
  /** アルゴリズム名のリスト */
  algoNames: string[];
  /** アルゴリズムごとの色マップ */
  algoColors: Record<string, string>;
  /** 色変更時のコールバック */
  onColorChange: (algoName: string, color: string) => void;
}

/**
 * グラフコントロールコンポーネント
 * 高さスライダーとアルゴリズムの色ピッカーを提供
 */
export const ChartControls: React.FC<ChartControlsProps> = ({
  height,
  onHeightChange,
  algoNames,
  algoColors,
  onColorChange,
}) => {
  return (
    <div className={styles.controls}>
      <div className={styles.controlGroup}>
        <span className={styles.label}>Height:</span>
        <input
          type="range"
          min="300"
          max="800"
          step="50"
          value={height}
          onChange={(e) => onHeightChange(Number(e.target.value))}
          className={styles.rangeInput}
        />
        <span className={styles.rangeValue}>{height}px</span>
      </div>

      <div className={styles.colorPickerGroup}>
        <span className={styles.label}>
          <Settings size={14} /> Colors:
        </span>
        {algoNames.map((algoName) => (
          <div key={algoName} className={styles.colorItem}>
            <input
              type="color"
              value={algoColors[algoName] || "#000000"}
              onChange={(e) => onColorChange(algoName, e.target.value)}
              className={styles.colorInput}
            />
            <span className={styles.colorName} title={algoName}>
              {algoName}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
