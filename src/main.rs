use tenbin::{Func, InstantBench, ScalingBench};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 計測したいターゲット関数（クロージャ）を定義
    // 計算だけでなく、Vecへの格納（Memory Alloc）が発生する処理です。
    let prime_finder = |n: &usize| {
        let n = *n;
        let mut primes = Vec::new();
        for i in 2..n {
            if (2..((i as f64).sqrt() as usize + 1)).all(|d| i % d != 0) {
                primes.push(i);
            }
        }
        primes // 戻り値 R
    };

    // 2. Func構造体を生成（関数ごとの管理単位）
    let func_bench = Func::new("prime_calculation_v1")
        .with_description("エラトステネスの篩を使わない素朴な素数判定の計測");

    // 3. 【Instant】代表値のベンチマークを追加
    // 「とりあえず1000の時はどうなの？」を測る
    let func_bench = func_bench.add_bench(InstantBench::new("baseline_1000", 1000, prime_finder));

    let scale = ScalingBench::new("growth_analysis", vec![10, 100, 1000, 5000], prime_finder);

    // 4. 【Scaling】スケーリングのベンチマークを追加
    // 入力が増えた時の「耐性」を測る
    let mut func_bench = func_bench.add_bench(scale);

    // 5. 実行してJSONに保存！
    // 内部で run_all() が呼ばれ、target/tenbin/prime_calculation_v1/ に保存されます。
    func_bench.run_and_save()?;

    println!("✅ 計測完了！ JSONレポートが生成されました。");
    Ok(())
}
