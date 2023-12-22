use multimarkov::MultiMarkov;
use regex::Regex;
use crate::clusterchain::builder::ClusterChainGeneratorBuilder;
use crate::RandomTextGenerator;

/// This struct, once trained on a corpus of training data, can be used repeatedly to generate
/// random text strings (i.e. names) that sort-of resemble the training data.  At its heart is a
/// Markov chain model.  The key difference between this struct and its cousin `CharacterChainGenerator`
/// is that this one learns vowel and consonant *clusters* and the relative probabilities with which
/// one cluster follows another.  So, from a string like `"fascinating"` it learns the following transitions:
///
/// - `"f"` or `"n"` -> `"a"`
/// - `"a"` -> `"sc"` or `"t"`
/// - `"sc"` or `"t"` -> `"i"`
/// - `"i"` -> `"n"` or `"ng"`
///
/// Random text will be procedurally generated by combining vowel and consonant clusters (which may
/// be individual letters or groups of letters) in similar frequencies to what was observed in the
/// training data.
///
/// Create an instance using the builder pattern:
/// ```
/// use markov_namegen::ClusterChainGenerator;
/// let dwarf_names = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();
/// let namegen = ClusterChainGenerator::builder().train(dwarf_names).build();
/// ```
///
/// Training data can be an iterator of `String` or of `&str` type, and you can call `.train()`
/// repeatedly, for cumulative training on more than one dataset.
///
/// Here's an example with all the optional settings:
///
/// ```
/// use markov_namegen::ClusterChainGenerator;
/// let pokedex_names = vec!["bulbasaur","charmander","squirtle","pikachu"].into_iter();
/// let mut namegen = ClusterChainGenerator::builder()
///     .with_order(2)
///     .with_prior(0.007)
///     .with_pattern("^[A-Za-z]{4,8}$")
///     .train(pokedex_names)
///     .build();
/// ```
///
/// You can set a pattern to filter acceptable names; for example above we are requiring that
/// results must be 4 to 8 characters long.  ClusterChainGenerator will simply re-roll new names
/// until it finds one that matches.  Be careful: if you supply a difficult-to-match pattern,
/// name generation may be very slow; if you supply an impossible-to-match pattern, for example
/// one that requires characters not seen in the training data, you will get an infinite loop.
///
/// Here's a final example that reads names from a file (one name per line), builds up a
/// ClusterChainGenerator, and then spits out a few names:
///
/// ```
/// use std::fs::File;
/// use std::io::{BufReader, BufRead};
/// use markov_namegen::ClusterChainGenerator;
/// use markov_namegen::RandomTextGenerator;
///
/// let file = File::open("resources/romans.txt").unwrap();
/// let reader = BufReader::new(file);
/// let lines = reader.lines().map(|l| l.unwrap() );
///
/// let mut namegen = ClusterChainGenerator::builder()
///     .train(lines)
///     .build();
///
/// for _i in 0..10 {
///     println!("{}", namegen.generate_one());
/// }
/// ```
///
pub struct ClusterChainGenerator<'a> {
    pub(super) model: MultiMarkov<String>,
    pub(super) pattern: Option<&'a str>,
}

impl<'a> ClusterChainGenerator<'a> {
    pub const DEFAULT_ORDER: i32 = 3;
    pub const DEFAULT_PRIOR: f64 = 0.001;

    pub fn builder() -> ClusterChainGeneratorBuilder<'a> {
        ClusterChainGeneratorBuilder::new()
    }

    fn generate_string(&mut self) -> String {
        // start with the beginning-of-word character
        let mut name = vec!["#".to_string()];
        name.push(self.model.random_next(&name).unwrap());
        while !name.ends_with(&*vec!["#".to_string()]) {
            // keep adding letters until we reach the end-of-word character
            name.push(self.model.random_next(&name).unwrap());
        }
        // remove the trailing and leading "#" signs
        name.pop();
        name.remove(0);
        let stringname = name.join("");
        stringname
    }
}

impl RandomTextGenerator for ClusterChainGenerator<'_> {
    fn generate_one(&mut self) -> String {
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
