use std::rc::Rc;

use poker_evaluator::Evaluator;

use poker_abstraction::histogram::{avg, emd, mse, Histogram};
use poker_abstraction::k_means::k_means;
use poker_abstraction::tables;

fn make_flops(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<usize> {
    println!("Getting Flops");

    let flop: Vec<Histogram> = tables::get(
        &(path.clone() + "flop.bin"),
        Box::new({
            let input = Rc::clone(strength);
            move || tables::generate_flop_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()).norm())
    .collect();

    println!("Clustering Flops");

    k_means(count, 15, &flop, avg, emd)
}

fn make_turns(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<usize> {
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

    k_means(count, 15, &turn, avg, emd)
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

    k_means(count, 75, &ochs, avg, emd)
}

fn make_rivers(
    count: usize,
    path: &String,
    evaluator: &Rc<Evaluator>,
    ochs: &Rc<Vec<usize>>,
) -> Vec<usize> {
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

    k_means(count, 15, &river, avg, mse)
}

pub fn main() {
    let evaluator = Rc::new(Evaluator::new("data/evaluator".to_string(), true));

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

    let _turns = tables::get(
        &(file.clone() + "turns.bin"),
        Box::new({
            let strength = Rc::clone(&strength);
            let path = path.clone();
            move || make_turns(2197, &path, &strength)
        }),
    );

    let ochs = Rc::new(tables::get(
        &(file.clone() + "ochs.bin"),
        Box::new({
            let strength = Rc::clone(&strength);
            let path = path.clone();
            move || make_ochs(13, &path, &strength)
        }),
    ));

    let _river = tables::get(
        &(file.clone() + "rivers.bin"),
        Box::new({
            let evaluator = Rc::clone(&evaluator);
            let path = path.clone();
            move || make_rivers(2197, &path, &evaluator, &ochs)
        }),
    );
}
