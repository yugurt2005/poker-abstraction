use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::*;


pub fn main() {
    let evaluator = Rc::new(Evaluator::new("data/evaluator".to_string()));

    let path: String = "data/histograms/".to_string();

    let strength = Rc::new(tables::get_strengths(
        path.clone() + "strength.bin",
        &evaluator,
    ));

    let file: String = "data/tables/".to_string();

    let _flop = tables::get_flop_clusters(
        2197,
        file.clone() + "flop.bin",
        path.clone() + "flop.bin",
        &strength,
    );
    let _turn = tables::get_turn_clusters(
        2197,
        file.clone() + "turn.bin",
        path.clone() + "turn.bin",
        &strength,
    );
    let ochs = tables::get_ochs_clusters(
        13,
        file.clone() + "ochs.bin",
        path.clone() + "ochs.bin",
        &strength,
    );
    let _river = tables::get_river_clusters(
        2197,
        file.clone() + "river.bin",
        path.clone() + "river.bin",
        Rc::clone(&evaluator),
        Rc::new(ochs),
    );
}
