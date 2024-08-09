use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::*;

fn display_cards(deal: Vec<u64>) {
    let ranks = [
        "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A",
    ];

    let suits = ["♠", "♦", "♣", "♥"];

    for mut cards in deal {
        while cards != 0 {
            let card = 63 - cards.leading_zeros();
            let rank = card % 13;
            let suit = card / 13;

            print!("{}{} ", ranks[rank as usize], suits[suit as usize]);

            cards &= !(1 << card);
        }
        print!("| ");
    }
    println!();
}

pub fn main() {
    let evaluator = Rc::new(Evaluator::new("data/evaluator".to_string()));

    let path: String = "data/histograms/".to_string();

    let strength = Rc::new(tables::get_strengths(
        path.clone() + "strength.bin",
        &evaluator,
    ));

    let file: String = "data/tables/".to_string();

    let flop = tables::get_flop_clusters(
        file.clone() + "flop.bin",
        path.clone() + "flop.bin",
        &strength,
    );
    let turn = tables::get_turn_clusters(
        file.clone() + "turn.bin",
        path.clone() + "turn.bin",
        &strength,
    );
    let ochs = tables::get_ochs_clusters(
        file.clone() + "ochs.bin",
        path.clone() + "ochs.bin",
        &strength,
    );
    let river = tables::get_river_clusters(
        file.clone() + "river.bin",
        path.clone() + "river.bin",
        Rc::clone(&evaluator),
        Rc::new(ochs),
    );
}
