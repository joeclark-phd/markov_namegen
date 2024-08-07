use markov_namegen::{CharacterChainGenerator, ClusterChainGenerator, RandomTextGenerator};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {

    // initialize logging
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    // Test of CharacterChainGenerator
    println!("Ten Roman names from CharacterChainGenerator:\n");

    let file = File::open("resources/romans.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.unwrap());

    let mut namegen = CharacterChainGenerator::builder()
        .with_order(3)
        .with_prior(0.007)
        //.with_pattern("^[a-z]*a$") // names ending with "a" (feminine names)
        .with_pattern("^[A-Za-z]{4,8}$") // names 4-8 characters long
        .with_rng(Box::new(SmallRng::seed_from_u64(123)))
        .train(lines)
        .build();

    for _i in 0..10 {
        println!("{}", namegen.generate_one());
    }

    // Test of ClusterChainGenerator
    println!("\nTen Roman names from ClusterChainGenerator:\n");

    let file2 = File::open("resources/romans.txt").unwrap();
    let reader2 = BufReader::new(file2);
    let lines2 = reader2.lines().map(|l| l.unwrap());

    let mut namegen2 = ClusterChainGenerator::builder()
        .with_order(3)
        .with_prior(0.0005)
        //.with_pattern("^[a-z]*a$") // names ending with "a" (feminine names)
        .with_pattern("^[A-Za-z]{4,8}$") // names 4-8 characters long
        .with_rng(Box::new(SmallRng::seed_from_u64(123)))
        .train(lines2)
        .build();

    for _i in 0..10 {
        println!("{}", namegen2.generate_one());
    }
}
