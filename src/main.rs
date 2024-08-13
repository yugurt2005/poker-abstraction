use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::*;


pub fn main() {
    let evaluator = Rc::new(Evaluator::new("data/evaluator".to_string()));

    let path: String = "data/histograms/".to_string();

    let strength = Rc::new(get_strengths(
        path.clone() + "strength.bin",
        &evaluator,
    ));

    let file: String = "data/tables/".to_string();

    let _flop = get_flop_clusters(
        file.clone() + "flop.bin",
        path.clone() + "flop.bin",
        &strength,
    );
    let _turn = get_turn_clusters(
        file.clone() + "turn.bin",
        path.clone() + "turn.bin",
        &strength,
    );
    let ochs = get_ochs_clusters(
        file.clone() + "ochs.bin",
        path.clone() + "ochs.bin",
        &strength,
    );
    let _river = get_river_clusters(
        file.clone() + "river.bin",
        path.clone() + "river.bin",
        Rc::clone(&evaluator),
        Rc::new(ochs),
    );
}
