use tenbin::{Bench, Func};

fn main() {
    Func::new("Deduplication_Battle")
        .with_description(
            "配列から重複を取り除く3つの手法における、CPU時間とメモリ確保のトレードオフ比較",
        )
        // 選手1：初心者がよくやる直感的な実装 (O(N^2))
        // 毎回 contains で全探索するため、要素が増えると急激に遅くなる。
        .add_function("Naive_Contains", |n: &usize| {
            // ダミーデータ生成（0〜99の数字が繰り返し入る配列）
            let data: Vec<usize> = (0..*n).map(|x| x % 100).collect();

            let mut unique = Vec::new();
            for item in data {
                if !unique.contains(&item) {
                    unique.push(item);
                }
            }
            unique
        })
        // 選手2：標準的なハッシュセットを使った実装 (O(N))
        // 速度は非常に速いが、内部でハッシュテーブルのメモリ確保(alloc)が複数回発生する。
        .add_function("HashSet", |n: &usize| {
            let data: Vec<usize> = (0..*n).map(|x| x % 100).collect();

            use std::collections::HashSet;
            let mut set = HashSet::new();
            for item in data {
                set.insert(item);
            }
            let unique: Vec<usize> = set.into_iter().collect();
            unique
        })
        // 選手3：Rustのプロが好むソート＆デダップ (O(N log N))
        // HashSetより計算量は少し多いが、メモリの追加確保（alloc）がゼロで済むため、
        // キャッシュ効率が極めて高く、実用上最速になることが多い。
        .add_function("Sort_and_Dedup", |n: &usize| {
            let data: Vec<usize> = (0..*n).map(|x| x % 100).collect();

            let mut unique = data; // 所有権を移動するだけ（追加メモリ不要）
            unique.sort_unstable(); // メモリを確保しない高速なソート
            unique.dedup(); // 隣り合う重複を削除
            unique
        })
        // 【競技登録】
        // 1. まずは N=1000 の時の「代表値」を見る
        .add_bench("baseline_1k", "N=1000の基本性能", Bench::Instant(1000))
        // 2. データが爆発したときにどうなるかの「スケーリング」を見る
        .add_bench(
            "scaling_stress",
            "データ量増大による計算量とアロケーションの推移",
            Bench::Scaling(vec![100, 1000, 5000, 10000, 50000]),
        )
        // 実行して保存
        .run_and_save()
        .unwrap();
}
