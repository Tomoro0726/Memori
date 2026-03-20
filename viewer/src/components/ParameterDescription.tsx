import { useState } from "react";
import styles from "./ParameterDescription.module.css";

const PARAMETERS = [
  {
    key: "cycles",
    desc: "CPUサイクル数。Linuxでは perf_event/RDTSC、x86_64環境ではRDTSCで計測。",
  },
  { key: "instructions", desc: "実行命令数。Linux環境でのみ取得可能。" },
  {
    key: "timeNs",
    desc: "実行時間（ナノ秒）。全環境で取得可能（real_time有効時）。",
  },
  { key: "allocCount", desc: "関数実行中のメモリ割り当て回数。" },
  { key: "allocBytes", desc: "割り当てられたメモリ総量（バイト）。" },
  { key: "deallocBytes", desc: "解放されたメモリ総量（バイト）。" },
  {
    key: "netBytes",
    desc: "正味のメモリ増加量（allocBytes - deallocBytes）。",
  },
  { key: "deallocCount", desc: "関数実行中のメモリ解放回数。" },
  {
    key: "netBytes",
    desc: "正味のメモリ増加量（allocBytes - deallocBytes）。Rust側で計算済み。",
  },
];

const ENV_DIFFS = [
  {
    os: "Linux",
    support: "すべて取得可能 (cycles / instructions / timeNs / メモリ)",
  },
  {
    os: "Windows / macOS (x86_64)",
    support: "cycles / timeNs / メモリ取得可能。instructionsは不可",
  },
  {
    os: "ARM Mac (Apple Silicon) 等",
    support: "timeNs / メモリのみ取得可能。cyclesは 0 と表示",
  },
];

export function ParameterDescription() {
  const [open, setOpen] = useState(false);

  return (
    <div className={styles.container}>
      <button
        className={styles.toggle}
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <span className={styles.icon}>{open ? "−" : "+"}</span>
        パラメーターと計測環境の詳細
      </button>

      {open && (
        <div className={styles.content}>
          <section className={styles.section}>
            <h4 className={styles.sectionTitle}>Metrics Definition</h4>
            <dl className={styles.list}>
              {PARAMETERS.map((p) => (
                <div key={p.key} className={styles.item}>
                  <dt className={styles.term}>{p.key}</dt>
                  <dd className={styles.description}>{p.desc}</dd>
                </div>
              ))}
            </dl>
          </section>

          <section className={styles.section}>
            <h4 className={styles.sectionTitle}>Environment Support</h4>
            <ul className={styles.envList}>
              {ENV_DIFFS.map((env) => (
                <li key={env.os}>
                  <strong>{env.os}:</strong> {env.support}
                </li>
              ))}
            </ul>
          </section>

          <p className={styles.footerNote}>
            ※ メモリ統計は GlobalAlloc
            をラップしたカスタムアロケーターにより、OSを問わず正確に追跡されます。
          </p>
        </div>
      )}
    </div>
  );
}
