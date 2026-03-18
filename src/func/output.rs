use crate::bench::BenchResult;
use crate::{Func, Measurement};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// --- JSON出力用のデータ構造 ---

/// main.json: その関数（フォルダ）の目次となるメタデータ
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub description: Option<String>,
    pub benchmarks: Vec<BenchMetadata>,
}

/// main.json内で、各ベンチマークの定義を記録
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchMetadata {
    pub name: String,
    pub description: Option<String>,
    pub pattern: String,
}

/// 履歴JSON: 各計測実行時の具体的なデータ
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonReport<I> {
    pub pattern: String,
    pub description: Option<String>,
    pub data: Vec<BenchJsonEntry<I>>,
}

/// 履歴JSON: 個々の計測ポイント（入力値と結果のペア）
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonEntry<I> {
    pub input: I,
    pub measurement: Measurement,
}

// --- Funcの実装改良 ---

impl<I> Func<I>
where
    I: Clone + serde::Serialize,
{
    /// ベンチマークを実行し、結果を履歴ファイルとメタデータ(main.json)に保存します。
    pub fn run_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. フォルダの準備
        let base_path = PathBuf::from("target/tenbin").join(&self.name);
        fs::create_dir_all(&base_path)?;

        // 2. メタデータの構築と main.json の更新
        let current_benchmarks: Vec<BenchMetadata> = self
            .benches
            .iter()
            .map(|b| BenchMetadata {
                name: b.name().clone(),
                description: b.description().cloned(),
                pattern: match b.pattern() {
                    crate::bench::BenchPattern::Instant => "instant".to_string(),
                    crate::bench::BenchPattern::Scaling => "scaling".to_string(),
                },
            })
            .collect();

        self.update_main_json(&base_path, current_benchmarks)?;

        // 3. 全ベンチマークを実行
        let raw_results = self.run_all();

        // 4. 履歴用JSONデータの構築
        let mut report_map = BTreeMap::new();
        for (i, result) in raw_results.into_iter().enumerate() {
            let bench_ref = &self.benches[i];

            let (pattern, items) = match result {
                BenchResult::Instant((input, m)) => (
                    "instant",
                    vec![BenchJsonEntry {
                        input,
                        measurement: m,
                    }],
                ),
                BenchResult::Scaling(list) => (
                    "scaling",
                    list.into_iter()
                        .map(|(input, m)| BenchJsonEntry {
                            input,
                            measurement: m,
                        })
                        .collect(),
                ),
            };

            report_map.insert(
                bench_ref.name().clone(),
                BenchJsonReport {
                    pattern: pattern.to_string(),
                    description: bench_ref.description().cloned(), // 履歴にも説明を載せる
                    data: items,
                },
            );
        }

        // 5. 履歴ファイルの保存 (001_YYYY-MM-DD.json)
        let next_num = self.get_next_file_number(&base_path);
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let history_path = base_path.join(format!("{:03}_{}.json", next_num, date_str));

        let json_data = serde_json::to_string_pretty(&report_map)?;
        fs::write(history_path, json_data)?;

        Ok(())
    }

    /// メタデータを比較し、変更がある場合のみ main.json を更新します。
    fn update_main_json(
        &self,
        path: &Path,
        benchmarks: Vec<BenchMetadata>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let main_path = path.join("main.json");
        let current_meta = FuncMetadata {
            name: self.name.clone(),
            description: self.description.clone(),
            benchmarks,
        };

        let should_write = if main_path.exists() {
            let content = fs::read_to_string(&main_path)?;
            if let Ok(existing_meta) = serde_json::from_str::<FuncMetadata>(&content) {
                // 名前、説明、またはベンチマーク構成が変わっていれば更新
                existing_meta != current_meta
            } else {
                true
            }
        } else {
            true
        };

        if should_write {
            let json = serde_json::to_string_pretty(&current_meta)?;
            fs::write(main_path, json)?;
        }
        Ok(())
    }

    /// フォルダをスキャンして次の連番を取得
    fn get_next_file_number(&self, path: &Path) -> u32 {
        fs::read_dir(path)
            .map(|dir| {
                dir.flatten()
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()?
                            .split('_')
                            .next()?
                            .parse::<u32>()
                            .ok()
                    })
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1)
    }
}
