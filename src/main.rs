use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::tables;


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

    println!("Getting Strengths");

    let strength = Rc::new(tables::get(
        &(path.clone() + "strengths.bin"),
        Box::new({
            let input = Rc::clone(&evaluator);
            move || tables::build_strengths(&input)
        }),
    ));

    let file: String = "data/tables/".to_string();

    let _flops = tables::get(
        &(file.clone() + "flops.bin"),
        Box::new({
            let strength = Rc::clone(&strength);
            let path = path.clone();
            move || tables::make_flops(2197, &path, &strength)
        }),
    );

    // let _turns = tables::get(
    //     &(file.clone() + "turns.bin"),
    //     Box::new({
    //         let strength = Rc::clone(&strength);
    //         let path = path.clone();
    //         move || tables::make_turns(2197, &path, &strength)
    //     }),
    // );

    // let ochs = Rc::new(tables::get(
    //     &(file.clone() + "ochs.bin"),
    //     Box::new({
    //         let strength = Rc::clone(&strength);
    //         let path = path.clone();
    //         move || tables::make_ochs(13, &path, &strength)
    //     }),
    // ));

    // let _river = tables::get(
    //     &(file.clone() + "rivers.bin"),
    //     Box::new({
    //         let evaluator = Rc::clone(&evaluator);
    //         let path = path.clone();
    //         move || tables::make_rivers(2197, &path, &evaluator, &ochs)
    //     }),
    // );
}
