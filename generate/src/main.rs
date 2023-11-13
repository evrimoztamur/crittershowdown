use std::collections::BTreeMap;

use rayon::prelude::*;
use shared::{Board, Level, Mage, Team};

fn generate_levels() -> Vec<Level> {
    (0..25)
        .map(|i| {
            let board = Board::new(4, 4).unwrap();

            let mages = vec![
                Mage::new(
                    0,
                    Team::Red,
                    (i as usize % 5).into(),
                    shared::Position(0, 0),
                ),
                Mage::new(
                    1,
                    Team::Red,
                    ((i + 1) as usize % 5).into(),
                    shared::Position(1, 0),
                ),
                Mage::new(
                    2,
                    Team::Blue,
                    (i as usize / 5).into(),
                    shared::Position(2, 3),
                ),
                Mage::new(
                    3,
                    Team::Blue,
                    ((i + 1) as usize / 5).into(),
                    shared::Position(3, 3),
                ),
            ];
            Level::new(board, mages, BTreeMap::default(), Team::Red)
        })
        .collect()
}

fn main() {
    let seed = 1;
    const N: usize = 100;

    // for level in generate_levels() {
    //     let simulations = Level::simulate(level, N, seed);

    //     println!(
    //         "{:?}",
    //         simulations
    //             .par_iter()
    //             .map(|game| { (game.turns(), game.evaluate()) })
    //             .fold(|| (0, 0), |(aa, ba), (ab, bb)| (aa + ab, ba + bb))
    //     );
    // }

    generate_levels()
        .par_iter()
        .map(|level| {
            let simulations = Level::simulate(level, N, seed);
            let result = simulations
                .par_iter()
                .map(|game| {
                    // println!("  {:?}", (game.turns(), game.evaluate()));
                    (game.turns(), game.evaluate())
                })
                .reduce(|| (0, 0), |(aa, ba), (ab, bb)| (aa + ab, ba + bb));

            println!("{}, {}, {}", level.as_code(), result.0, result.1);
        })
        .count();
}
