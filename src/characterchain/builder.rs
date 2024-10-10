use crate::characterchain::generator::CharacterChainGenerator;
use multimarkov::builder::MultiMarkovBuilder;
use multimarkov::MultiMarkov;
use rand::RngCore;
use regex::Regex;
use std::ops::Deref;

/// A Builder pattern for CharacterChainGenerator.
pub struct CharacterChainGeneratorBuilder<'a> {
    model: MultiMarkovBuilder<char>,
    pattern: Option<&'a str>,
}

impl<'a> CharacterChainGeneratorBuilder<'a> {
    /// Instantiate a new builder with default values.
    pub fn new() -> Self {
        Self {
            model: MultiMarkov::<char>::builder()
                .with_order(CharacterChainGenerator::DEFAULT_ORDER)
                .with_prior(CharacterChainGenerator::DEFAULT_PRIOR),
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
        assert!(order > 0, "Order must be an integer greater than zero.");
        self.model = self.model.with_order(order); // update model now, so it'll affect training
        self
    }
    /// Sets a custom value for prior probabilities. The greater the prior, the more likely you'll
    /// see character combinations that do NOT occur in the training data.
    ///
    /// The way this works is, each observed transition gets a score/weight of 1.0 every time it's
    /// observed.  These are never normalized or turned into percentages, so if your training set
    /// is larger, typical weights will be larger. A prior of 0.1 will make an unobserved transition
    /// occur as frequently as if it had been seen 1/10 as often as a transition observed once in
    /// the training data.  That may not seem like much, but depending on the size of your alphabet
    /// there might be *a lot* of these, adding up to quite a lot of weird, unexpected transitions.
    /// Small values, such as 0.001 to 0.01, are recommended.  Tweak until you get the right amount
    /// of randomness for your application.
    ///
    /// By default, they are set to `CharacterChainGenerator::DEFAULT_PRIOR`.
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
    pub fn with_rng(mut self, rng: Box<dyn RngCore + Send + Sync>) -> Self {
        self.model = self.model.with_rng(rng);
        self
    }
    /// Ingest a training data set to train the model.
    /// The argument 'sequences' is an iterator of either `String` or `&str` values, the words or names
    /// that we want our randomly generated text to resemble.
    pub fn train(mut self, sequences: impl Iterator<Item = impl Deref<Target = str>>) -> Self {
        self.model = self.model.train(
            sequences
                .map(|s| s.to_lowercase()) // lowercase the input
                .map(|mut s| {
                    s.insert(0, '#');
                    s.push('#');
                    s
                }) // add the beginning-of-character and end-of-character strings
                .map(|s| s.chars().collect()), // turn the input stream into an iterator of Vec<char>
        );
        self
    }
    /// Build the CharacterChainGenerator (consuming the "Builder" in the process).
    pub fn build(self) -> CharacterChainGenerator {
        let pattern = self.pattern.map(|pat| Regex::new(pat).unwrap());
        CharacterChainGenerator {
            model: self.model.build(),
            pattern,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CharacterChainGenerator;

    #[test]
    fn test_builder_pattern_works() {
        let _generator = CharacterChainGenerator::builder()
            .with_order(2)
            .with_prior(0.007)
            .with_pattern("foo")
            .build();
    }

    #[test]
    #[should_panic(expected = "Order must be an integer greater than zero.")]
    fn test_order_cannot_be_less_than_one() {
        let _generator = CharacterChainGenerator::builder().with_order(0).build();
    }

    #[test]
    fn test_can_train_model_with_vec_of_strings() {
        // Training works equally well with an iterator of Strings or an iterator of &strs.
        let inputs = vec![
            "dopey", "sneezy", "bashful", "sleepy", "happy", "grumpy", "doc",
        ]
        .into_iter();
        //let inputs_as_strings = vec![String::from("dopey"),String::from("sneezy"),String::from("bashful"),String::from("sleepy"),String::from("happy"),String::from("grumpy"),String::from("doc")].into_iter();
        let _generator = CharacterChainGenerator::builder().train(inputs).build();
    }
}
