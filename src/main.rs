use rand::Rng;
use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::histogram::Histogram;
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

    let _turns = tables::get(
        &(file.clone() + "turns.bin"),
        Box::new({
            let strength = Rc::clone(&strength);
            let path = path.clone();
            move || tables::make_turns(2197, &path, &strength)
        }),
    );

    let ochs = Rc::new(tables::get(
        &(file.clone() + "ochs.bin"),
        Box::new({
            let strength = Rc::clone(&strength);
            let path = path.clone();
            move || tables::make_ochs(13, &path, &strength)
        }),
    ));

    println!("{:?}", ochs);

    let _turns = tables::get(
        &(file.clone() + "turns.bin"),
        Box::new({
            let input = Rc::clone(&strength);
            let path = path.clone();
            move || tables::make_turns(2197, &path, &input)
        }),
    );

    let indexer = poker_indexer::Indexer::new(vec![2, 4]);

    let mut counts: Vec<u32> = vec![0; 2197];

    for f in _turns.iter() {
        counts[*f as usize] += 1;
    }

    counts.sort();
    counts.reverse();

    println!("{:?}", counts[0..50].to_vec());

    let res: Vec<(Vec<u64>, usize)> = _turns
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| {
            if x == 11 {
                Some((indexer.unindex(i as u32, 1).into_vec(), i))
            } else {
                None
            }
        })
        .collect();

    let turn: Vec<Histogram> = tables::get(
            &(path.clone() + "turn.bin"),
            Box::new({
                let input = Rc::clone(&strength);
                move || tables::generate_turn_histograms(&input)
            }),
        )
        .into_iter()
        .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()))
        .collect();

    let mut rng = rand::thread_rng();

    println!("{}", res.len());
    for i in 0..10 {
        let i = rng.gen_range(0..res.len());
        display_cards(res[i].0.clone());
        turn[res[i].1].display();
    }
}
