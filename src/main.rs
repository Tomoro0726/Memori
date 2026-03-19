use tenbin::{Bench, Func};

fn main() {
    Func::new("Set_Performance")
        .with_description("HashSetとBTreeSetの性能比較")
        // 【選手登録】計測したい関数をいくつでも追加
        .add_function("HashSet", |n| {
            let mut s = std::collections::HashSet::new();
            for i in 0..*n {
                s.insert(i);
            }
        })
        .add_function("BTreeSet", |n| {
            let mut s = std::collections::BTreeSet::new();
            for i in 0..*n {
                s.insert(i);
            }
        })
        // 【競技登録】代表値（1点）とスケーリング（複数点）を組み合わせ自由で追加
        .add_bench("baseline", "標準的な負荷", Bench::Instant(1000))
        .add_bench(
            "stress_test",
            "負荷増大時の推移",
            Bench::Scaling(vec![10, 100, 1000, 5000]),
        )
        // 実行！
        .run_and_save()
        .unwrap();
}
