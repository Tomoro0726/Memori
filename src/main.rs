use std::hint::black_box;

use kasane_logic::{Coordinate, Geometry, Triangle};
use tenbin::{Bench, Func};

fn main() {
    let mut func = Func::new("Triangle関数の可視化")
        .add_bench(
            "Z",
            "ズームレベルが大きければ大きいほど、処理に時間がかかる。",
            Bench::Scaling(vec![18, 19, 20, 21, 22, 23, 24]),
        )
        .add_function("通常の三角形関数", |z| {
            let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
            let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();
            let point_c = Coordinate::new(35.680700, 139.767050, 5.0).unwrap();

            let triangle = Triangle::new([tokyo_station, point_b, point_c]);

            let iter = triangle
                .single_ids(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        })
        .add_function("地球の歪みを考慮しない関数", |z| {
            let tokyo_station = Coordinate::new(35.681000, 139.767000, 0.0).unwrap();
            let point_b = Coordinate::new(35.681200, 139.767200, 10.0).unwrap();
            let point_c = Coordinate::new(35.680700, 139.767050, 5.0).unwrap();

            let triangle = Triangle::new([tokyo_station, point_b, point_c]);

            let iter = triangle
                .single_ids_limited(*z as u8)
                .expect("Failed to get iterator");
            black_box(iter.count())
        });

    func.run_and_save().unwrap();
}
