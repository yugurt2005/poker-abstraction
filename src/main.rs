use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::histogram::{agg, emd, mse, Histogram};
use poker_abstraction::k_means::k_means;
use poker_abstraction::tables;

fn make_flops(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    println!("Getting Flops");

    let flop: Vec<Histogram> = tables::get(
        &(path.clone() + "flop.bin"),
        Box::new({
            let input = Rc::clone(strength);
            move || tables::generate_flop_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()))
    .collect();

    println!("Clustering Flops");

    k_means(count, 1, &flop, agg, emd)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

fn make_turns(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    println!("Getting Turns");

    let turn: Vec<Histogram> = tables::get(
        &(path.clone() + "turn.bin"),
        Box::new({
            let input = Rc::clone(strength);
            move || tables::generate_turn_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()).norm())
    .collect();

    println!("Clustering Turns");

    k_means(count, 1, &turn, agg, emd)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

fn make_ochs(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<usize> {
    println!("Getting OCHS");

    let ochs: Vec<Histogram> = tables::get(
        &(path.clone() + "ochs.bin"),
        Box::new({
            let input = Rc::clone(strength);
            move || tables::build_ochs_histograms(&input)
        }),
    );

    println!("Clustering OCHS");

    k_means(count, 75, &ochs, agg, emd)
}

fn make_rivers(
    count: usize,
    path: &String,
    evaluator: &Rc<Evaluator>,
    ochs: &Rc<Vec<usize>>,
) -> Vec<u16> {
    println!("Getting Rivers");

    let river: Vec<Histogram> = tables::get(
        &(path.clone() + "river.bin"),
        Box::new({
            let evaluator = Rc::clone(&evaluator);
            let ochs = Rc::clone(&ochs);
            move || tables::generate_river_histograms(&evaluator, &ochs)
        }),
    );

    println!("Clustering Rivers");

    k_means(count, 1, &river, agg, mse)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

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
            move || make_flops(2197, &path, &strength)
        }),
    );

    let indexer = poker_indexer::Indexer::new(vec![2, 3]);

    let mut counts: Vec<u32> = vec![0; 2197];

    for f in _flops.iter() {
        counts[*f as usize] += 1;
    }

    println!("{:?}", counts[0..50].to_vec());

    let res: Vec<(Vec<u64>, usize)> = _flops
        .iter()
        .enumerate()
        .filter_map(|(i, &x)| {
            if x == 1 {
                Some((indexer.unindex(i as u32, 1).into_vec(), i))
            } else {
                None
            }
        })
        .collect();

    let flop: Vec<Histogram> = tables::get(
        &(path.clone() + "flop.bin"),
        Box::new({
            let input = Rc::clone(&strength);
            move || tables::generate_flop_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()).norm())
    .collect();

    println!("{}", res.len());
    for i in 0..10 {
        display_cards(res[i].0.clone());
        flop[res[i].1].display();
    }

    // let _turns = tables::get(
    //     &(file.clone() + "turns.bin"),
    //     Box::new({
    //         let strength = Rc::clone(&strength);
    //         let path = path.clone();
    //         move || make_turns(2197, &path, &strength)
    //     }),
    // );

    // let ochs = Rc::new(tables::get(
    //     &(file.clone() + "ochs.bin"),
    //     Box::new({
    //         let strength = Rc::clone(&strength);
    //         let path = path.clone();
    //         move || make_ochs(13, &path, &strength)
    //     }),
    // ));

    // let _river = tables::get(
    //     &(file.clone() + "rivers.bin"),
    //     Box::new({
    //         let evaluator = Rc::clone(&evaluator);
    //         let path = path.clone();
    //         move || make_rivers(2197, &path, &evaluator, &ochs)
    //     }),
    // );
}
