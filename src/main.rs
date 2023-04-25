mod characterchain;
mod interface;

use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::characterchain::generator::CharacterChainGenerator;
use crate::interface::RandomTextGenerator;

fn main() {

    let file = File::open("resources/romans.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.unwrap() );

    let namegen = CharacterChainGenerator::builder()
        .with_order(3)
        .with_prior(0.007)
//        .with_pattern("^[a-z]*a$") // names ending with "a" (feminine names)
        .with_pattern("^[A-Za-z]{4,8}$") // names 4-8 characters long
        .train(lines)
        .build();

    for _i in 0..10 {
        println!("{}", namegen.generate_one());
    }
}


