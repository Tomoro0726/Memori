use crate::Runner;
pub mod output;

/// 計測の入力パターン（代表値 or スケーリング）
pub enum Bench<I> {
    /// 単一の代表値で計測する
    Instant(I),
    /// 複数の値でスケーリング（推移）を計測する
    Scaling(Vec<I>),
}

/// 1つの計測シナリオ（パターン）
pub struct BenchPattern<I> {
    pub name: String,
    pub description: String,
    pub input: Bench<I>,
}

pub struct Func<I>
where
    I: Clone,
{
    name: String,
    description: Option<String>,
    /// 登録された複数の関数（ライバルたち）
    functions: Vec<(String, Box<dyn FnMut(&I)>)>,
    /// 登録された計測シナリオ
    patterns: Vec<BenchPattern<I>>,
}

impl<I> Func<I>
where
    I: Clone,
{
    /// 新しい計測スイートを作成します
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            functions: Vec::new(),
            patterns: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// 計測対象の関数を追加します（複数登録可能）
    pub fn add_function<F, R>(mut self, name: &str, mut func: F) -> Self
    where
        F: FnMut(&I) -> R + 'static,
    {
        // 戻り値 R を捨てて、型を統一する
        let wrapped = move |i: &I| {
            let _ = func(i);
        };
        self.functions.push((name.to_string(), Box::new(wrapped)));
        self
    }

    /// 計測シナリオ（代表値 or スケーリング）を追加します
    pub fn add_bench(mut self, name: &str, description: &str, input: Bench<I>) -> Self {
        self.patterns.push(BenchPattern {
            name: name.to_string(),
            description: description.to_string(),
            input,
        });
        self
    }

    /// 登録された全関数に対して、全パターンを実行します（マトリックス実行）
    pub fn run_all(&mut self) {
        println!("🚀 実行開始: {}", self.name);

        // 1. シナリオ（パターン）ごとに回す
        for pattern in &self.patterns {
            println!(" ├─ パターン: {} ({:?})", pattern.name, pattern.description);

            // 2. 登録された関数ごとに回す
            for (func_name, func) in &mut self.functions {
                println!(" │   ├─ 関数: {}", func_name);

                // 3. パターンの種類に応じて Runner を実行
                match &pattern.input {
                    Bench::Instant(val) => {
                        let mut runner = Runner::new(val.clone(), func);
                        let m = runner.run();
                        // ここで m (Measurement) を保存する処理
                    }
                    Bench::Scaling(vals) => {
                        for val in vals {
                            let mut runner = Runner::new(val.clone(), &mut *func);
                            let m = runner.run();
                            // ここで (val, m) のペアを保存する処理
                        }
                    }
                }
            }
        }
    }
}
