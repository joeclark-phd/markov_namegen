use std::ops::Deref;
use multimarkov::builder::MultiMarkovBuilder;
use multimarkov::MultiMarkov;
use rand::RngCore;
use crate::clusterchain::generator::ClusterChainGenerator;
use is_vowel::IsRomanceVowel;

/// A Builder pattern for ClusterChainGenerator.
pub struct ClusterChainGeneratorBuilder<'a> {
    model: MultiMarkovBuilder<String>,
    pattern: Option<&'a str>,
}

impl<'a> ClusterChainGeneratorBuilder<'a> {

    /// Instantiate a new builder with default values.
    pub fn new() -> Self {
        Self {
            model: MultiMarkov::<String>::builder()
                .with_order(ClusterChainGenerator::DEFAULT_ORDER)
                .with_prior(ClusterChainGenerator::DEFAULT_PRIOR),
            pattern: None,
        }
    }
    /// Sets a custom regex pattern for pattern matching (filtering) of output.
    /// The generator will generate names repeatedly until it finds one that matches your pattern.
    /// Be warned that if you define an impossible-to-match pattern (e.g. one that includes letters
    /// not found in the training dataset), you could end up with an infinite loop when you try
    /// to generate a name.
    pub fn with_pattern(mut self, pattern: &'a str) -> Self {
        self.pattern = Some(pattern);
        self
    }
    /// Sets a custom value for order of the Markov model.
    /// Must be an integer greater than zero.  Values from 1 to 3 are recommended.
    /// Higher-order models will make procedurally generated text more like the training data,
    /// and less random, and will be slower and require more memory.
    ///
    /// NOTE: Order should be set *before* training the model with `.train()`
    pub fn with_order(mut self, order: i32) -> Self {
        assert!(order > 0,"Order must be an integer greater than zero.");
        self.model = self.model.with_order(order); // update model now, so it'll affect training
        self
    }
    /// Sets a custom value for prior probabilities.
    /// The greater the prior, the more likely you'll see character combinations that do NOT occur in the training data.
    ///
    /// The way this works is, each observed transition gets a score/weight of 1.0 every time it's
    /// observed.  These are never normalized or turned into percentages, so if your training set
    /// is larger, typical weights will be larger. A prior of 0.1 will make an unobserved transition
    /// occur as frequently as if it had been seen 1/10 as often as a transition observed once in
    /// the training data.  That may not seem like much, but depending on the size of your alphabet
    /// there might be *a lot* of these, adding up to quite a lot of weird, unexpected transitions.
    ///
    /// You will want smaller values here than in CharacterChainGenerator, because there will be
    /// more clusters than there are characters in the alphabet.  0.0001 to 0.001 is recommended.
    /// Tweak until you get the right amount of randomness for your application.
    ///
    /// By default, they are set to `ClusterChainGenerator::DEFAULT_PRIOR`.
    pub fn with_prior(mut self, prior: f64) -> Self {
        self.model = self.model.with_prior(prior);
        self
    }
    /// Set the priors to None.
    pub fn without_prior(mut self) -> Self {
        self.model = self.model.without_prior();
        self
    }
    /// Sets a custom Random Number Generator (RNG) for the model.
    pub fn with_rng(mut self, rng: Box<dyn RngCore>) -> Self {
        self.model = self.model.with_rng(rng);
        self
    }
    /// Ingest a training data set to train the model.
    /// The argument 'sequences' is an iterator of either `String` or `&str` values, the words or names
    /// that we want our randomly generated text to resemble.
    pub fn train(mut self, sequences: impl Iterator<Item=impl Deref<Target = str>>) -> Self {
        self.model = self.model.train( sequences
                                           .map(|s| s.to_lowercase()) // lowercase the input
                                           .map(|s| ClusterChainGeneratorBuilder::clusterize(s))
                                           .map(|mut s| { s.insert(0, "#".to_string()); s.push("#".to_string()); s }) // add the beginning-of-character and end-of-character strings
        );
        self
    }

    /// Transforms a String into a Vec<String> of vowel and consonant clusters.
    /// It depends on the `is_vowel` crate, which only identifies vowels for romance languages.
    /// Thus, vowels like 'æ', 'œ', and 'ø' will be treated as consonants.
    /// Also, 'y' and 'w' are treated as consonants, in case you were wondering.
    fn clusterize(sequence: String) -> Vec<String> {
        let mut cluster_chain: Vec<String> = Vec::new();
        let mut chars = sequence.chars();
        let first_character = chars.nth(0).unwrap();
        // start the first cluster with the first character
        let mut current_cluster = String::from(first_character);
        // flag the type of the first cluster (vowel or consonant)
        let mut is_vowel_cluster = first_character.is_romance_vowel();
        // now loop through the other characters and build up the vec of clusters
        for c in chars {
            if c.is_romance_vowel() == is_vowel_cluster {
                // in other words, if the next char is of the same typ (vowel/consonant) as the last one(s), add it to the current cluster
                current_cluster.push(c);
            } else {
                // otherwise, add the current cluster to the vec and begin a new cluster with this character
                cluster_chain.push(current_cluster);
                current_cluster = String::from(c);
                is_vowel_cluster = !is_vowel_cluster;
            }
        }
        // finalize the final cluster by adding it to the list
        cluster_chain.push(current_cluster);
        cluster_chain
    }

    /// Build the ClusterChainGenerator (consuming the "Builder" in the process).
    pub fn build(self) -> ClusterChainGenerator<'a> {
        ClusterChainGenerator {
            model: self.model.build(),
            pattern: self.pattern,
        }
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use is_vowel::IsRomanceVowel;
    use crate::clusterchain::builder::ClusterChainGeneratorBuilder;
    use crate::clusterchain::generator::ClusterChainGenerator;

    #[test]
    fn test_is_vowel_crate_works() {
        // This crate may be "good enough" for now, but it doesn't cover vowels from non-romance languages (like æ, œ, and ø).
        // There is apparently no real standard for identifying vowels.
        // In the Java version of this, I didn't have a handy crate like is_vowel, so I implemented my
        // own solution based on a long list of unicode vowels which you can find here:
        // https://github.com/joeclark-phd/random-text-generators/blob/master/src/main/java/net/joeclark/proceduralgeneration/ClusterChainGenerator.java
        assert!('a'.is_romance_vowel());
        assert!(!'b'.is_romance_vowel());
        assert!(!'y'.is_romance_vowel());
        assert!('ĳ'.is_romance_vowel());
        let extra_vowels : HashSet<char> = "yæœøɏʎ".chars().collect();  // treat 'y' as a vowel, too (and some non-romance vowels)
        assert!('y'.is_romance_vowel_including(&extra_vowels));
        assert!('ǣ'.is_romance_vowel_including(&extra_vowels));
        assert!('ǿ'.is_romance_vowel_including(&extra_vowels));
    }

    #[test]
    fn test_clusterize() {
        assert_eq!(ClusterChainGeneratorBuilder::clusterize(String::from("foobar")),
                   vec!["f".to_string(),"oo".to_string(),"b".to_string(),"a".to_string(),"r".to_string()]);
    }

    #[test]
    fn test_builder_pattern_works() {
        let generator = ClusterChainGenerator::builder().with_order(2).with_prior(0.007).with_pattern("foo").build();
    }

    #[test]
    #[should_panic(expected="Order must be an integer greater than zero.")]
    fn test_order_cannot_be_less_than_one() {
        let generator = ClusterChainGenerator::builder().with_order(0).build();
    }

    #[test]
    fn test_can_train_model_with_vec_of_strings() {
        // Training works equally well with an iterator of Strings or an iterator of &strs.
        let inputs = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();
        let generator = ClusterChainGenerator::builder().train(inputs).build();
    }

}
