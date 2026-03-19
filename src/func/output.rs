use crate::{Func, Measurement};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// --- JSON出力用の構造体 ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonReport<I> {
    pub pattern_type: String, // "instant" または "scaling"
    pub description: String,
    // 関数名をキーにして、その計測結果の配列を保持する
    pub results: BTreeMap<String, Vec<BenchJsonEntry<I>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonEntry<I> {
    pub input: I,
    pub measurement: Measurement,
}

// 目次となる main.json 用
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub description: Option<String>,
    pub functions: Vec<String>,
    pub patterns: Vec<PatternMetadata>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatternMetadata {
    pub name: String,
    pub description: String,
    pub pattern_type: String,
}

impl<I> Func<I>
where
    I: Clone + serde::Serialize + 'static,
{
    /// 全パターン×全関数を実行し、結果をJSONとして保存します
    pub fn run_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = PathBuf::from("target/tenbin").join(&self.name);
        fs::create_dir_all(&base_path)?;

        // 1. main.json (メタデータ) の更新
        self.update_main_json(&base_path)?;

        // 保存用の大枠: Key = パターン名 (例: "stress_test")
        let mut report_map: BTreeMap<String, BenchJsonReport<I>> = BTreeMap::new();

        println!("🚀 実行開始: {}", self.name);

        // 2. パターン（シナリオ）ごとにループ
        for pattern in &self.patterns {
            println!(" ├─ パターン: {} ({:?})", pattern.name, pattern.description);

            let pattern_type = match &pattern.input {
                crate::Bench::Instant(_) => "instant",
                crate::Bench::Scaling(_) => "scaling",
            };

            // このパターンに紐づく、各関数の結果を保持するマップ
            let mut pattern_results = BTreeMap::new();

            // 3. 関数（ライバル）ごとにループ
            for (func_name, func) in &mut self.functions {
                println!(" │   ├─ 関数: {}", func_name);
                let mut data_entries = Vec::new();

                // 4. 計測エンジンの実行
                match &pattern.input {
                    crate::Bench::Instant(val) => {
                        // Box内のクロージャに可変参照を渡す
                        let mut runner = crate::runner::Runner::new(val.clone(), &mut **func);
                        let m = runner.run();
                        data_entries.push(BenchJsonEntry {
                            input: val.clone(),
                            measurement: m,
                        });
                    }
                    crate::Bench::Scaling(vals) => {
                        for val in vals {
                            let mut runner = crate::runner::Runner::new(val.clone(), &mut **func);
                            let m = runner.run();
                            data_entries.push(BenchJsonEntry {
                                input: val.clone(),
                                measurement: m,
                            });
                        }
                    }
                }

                // 取得したデータを関数名ごとに保存
                pattern_results.insert(func_name.clone(), data_entries);
            }

            // このパターンの全関数の結果をまとめる
            report_map.insert(
                pattern.name.clone(),
                BenchJsonReport {
                    pattern_type: pattern_type.to_string(),
                    description: pattern.description.clone(),
                    results: pattern_results,
                },
            );
        }

        // 5. 履歴ファイルの保存 (001_YYYY-MM-DD.json)
        let next_num = self.get_next_file_number(&base_path);
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let history_path = base_path.join(format!("{:03}_{}.json", next_num, date_str));

        let json_data = serde_json::to_string_pretty(&report_map)?;
        fs::write(history_path, json_data)?;

        println!("✅ 保存完了: {} に出力されました。", self.name);
        Ok(())
    }

    /// メタデータを最新化する
    fn update_main_json(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let main_path = path.join("main.json");
        let current_meta = FuncMetadata {
            name: self.name.clone(),
            description: self.description.clone(),
            functions: self
                .functions
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
            patterns: self
                .patterns
                .iter()
                .map(|p| PatternMetadata {
                    name: p.name.clone(),
                    description: p.description.clone(),
                    pattern_type: match p.input {
                        crate::Bench::Instant(_) => "instant".to_string(),
                        crate::Bench::Scaling(_) => "scaling".to_string(),
                    },
                })
                .collect(),
        };

        let should_write = if main_path.exists() {
            let content = fs::read_to_string(&main_path)?;
            if let Ok(existing) = serde_json::from_str::<FuncMetadata>(&content) {
                existing != current_meta
            } else {
                true
            }
        } else {
            true
        };

        if should_write {
            fs::write(main_path, serde_json::to_string_pretty(&current_meta)?)?;
        }
        Ok(())
    }

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
