use regex::Regex;
use crate::characterchain::builder::CharacterChainGeneratorBuilder;
use crate::interface::RandomTextGenerator;
use multimarkov::MultiMarkov;

/// This struct, once trained on a corpus of training data, can be used repeatedly to generate
/// random text strings (i.e. names) that sort-of resemble the training data.  At its heart is a
/// Markov chain model that keeps track of the relative probabilities with which
/// letters of the alphabet follow other letters in the training data set.
///
/// Create an instance using the builder pattern:
/// ```
/// use markov_namegen::CharacterChainGenerator;
/// let dwarf_names = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();
/// let namegen = CharacterChainGenerator::builder().train(dwarf_names).build();
/// ```
///
/// Training data can be an iterator of `String` or of `&str` type, and you can call `.train()`
/// repeatedly, for cumulative training on more than one dataset.
///
/// Here's an example with all the optional settings:
///
/// ```
/// use markov_namegen::CharacterChainGenerator;
/// let pokedex_names = vec!["bulbasaur","charmander","squirtle","pikachu"].into_iter();
/// let namegen = CharacterChainGenerator::builder()
///     .with_order(2)
///     .with_prior(0.007)
///     .with_pattern("^[A-Za-z]{4,8}$")
///     .train(pokedex_names)
///     .build();
/// ```
///
/// You can set a pattern to filter acceptable names; for example above we are requiring that
/// results must be 4 to 8 characters long.  CharacterChainGenerator will simply re-roll new names
/// until it finds one that matches.  Be careful: if you supply a difficult-to-match pattern,
/// name generation may be very slow; if you supply an impossible-to-match pattern, for example
/// one that requires characters not seen in the training data, you will get an infinite loop.
///
/// Here's a final example that reads names from a file (one name per line), builds up a
/// CharacterChainGenerator, and then spits out a few names:
///
/// ```
/// use std::fs::File;
/// use std::io::{BufReader, BufRead};
/// use markov_namegen::CharacterChainGenerator;
/// use markov_namegen::RandomTextGenerator;
///
/// let file = File::open("resources/romans.txt").unwrap();
/// let reader = BufReader::new(file);
/// let lines = reader.lines().map(|l| l.unwrap() );
///
/// let namegen = CharacterChainGenerator::builder()
///     .train(lines)
///     .build();
///
/// for _i in 0..10 {
///     println!("{}", namegen.generate_one());
/// }
/// ```
///
pub struct CharacterChainGenerator<'a> {
    pub(super) model: MultiMarkov<char>,
    pub(super) pattern: Option<&'a str>,
}

impl<'a> CharacterChainGenerator<'a> {
    pub const DEFAULT_ORDER: i32 = 3;
    pub const DEFAULT_PRIOR: f64 = 0.005;

    pub fn builder() -> CharacterChainGeneratorBuilder<'a> {
        CharacterChainGeneratorBuilder::new()
    }

    fn generate_string(&self) -> String {
        // start with the beginning-of-word character
        let mut name = vec!['#'];
        name.push(self.model.random_next(&name).unwrap());
        while !name.ends_with(&*vec!['#']) {
            // keep adding letters until we reach the end-of-word character
            name.push(self.model.random_next(&name).unwrap());
        }
        // remove the trailing and leading "#" signs
        name.pop();
        name.remove(0);
        let stringname = name.iter().collect::<String>();
        stringname
    }
}

impl RandomTextGenerator for CharacterChainGenerator<'_> {
    fn generate_one(&self) -> String {
        match self.pattern {
            None => self.generate_string(),
            Some(pattern) => {
                let re = Regex::new(pattern).unwrap();
                let mut candidate = self.generate_string();
                while !re.is_match(&*candidate) {
                    //println!("got '{}', re-rolling!", candidate);
                    candidate = self.generate_string();
                }
                candidate
            }
        }
    }
}
