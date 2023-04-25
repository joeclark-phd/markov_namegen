use std::ops::Deref;
use multimarkov::MultiMarkov;
use crate::characterchain::generator::CharacterChainGenerator;

/// A Builder pattern for CharacterChainGenerator.
pub struct CharacterChainGeneratorBuilder<'a> {
    model: MultiMarkov<char>,
    pattern: Option<&'a str>,
    order: i32,
    prior: f64,
}

impl<'a> CharacterChainGeneratorBuilder<'a> {
    /// Instantiate a new builder with default values.
    pub fn new() -> Self {
        Self {
            model: MultiMarkov::<char>::new(),
            pattern: None,
            order: CharacterChainGenerator::DEFAULT_ORDER,
            prior: CharacterChainGenerator::DEFAULT_PRIOR,
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
    pub fn with_order(mut self, order: i32) -> Self {
        assert!(order > 0,"Order must be an integer greater than zero.");
        self.order = order;
        self
    }
    /// Sets a custom value for prior probabilities.  Values from 0.001 to 0.01 are recommended.
    /// The greater the prior, the more likely you'll see character combinations that do NOT occur in the training data.
    /// By default, they are set to CharacterChainGenerator::DEFAULT_PRIOR.
    pub fn with_prior(mut self, prior: f64) -> Self {
        self.prior = prior;
        self
    }
    /// Set the priors to 0.0.
    pub fn without_prior(mut self) -> Self {
        self.prior = 0.0;
        self
    }
    /// Ingest a training data set to train the model.
    /// The argument 'sequences' is an iterator of either `String` or `&str` values, the words or names
    /// that we want our randomly generated text to resemble.
    pub fn train(mut self, sequences: impl Iterator<Item=impl Deref<Target = str>>) -> Self {
        self.model = self.model.train( sequences.map(|s| s.to_lowercase().chars().collect()) );
        self
    }
    /// Build the CharacterChainGenerator (consuming the "Builder" in the process).
    pub fn build(self) -> CharacterChainGenerator<'a> {
        CharacterChainGenerator {
            model: self.model.with_order(self.order).with_priors(self.prior).build(),
            pattern: self.pattern,
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::characterchain::generator::CharacterChainGenerator;

    #[test]
    fn test_builder_pattern_works() {
        let generator = CharacterChainGenerator::builder().with_order(2).with_prior(0.007).with_pattern("foo").build();
    }

    #[test]
    #[should_panic(expected="Order must be an integer greater than zero.")]
    fn test_order_cannot_be_less_than_one() {
        let generator = CharacterChainGenerator::builder().with_order(0).build();
    }

    #[test]
    fn test_can_train_model_with_vec_of_strings() {
        // Training works equally well with an iterator of Strings or an iterator of &strs.
        let inputs = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();
        //let inputs_as_strings = vec![String::from("dopey"),String::from("sneezy"),String::from("bashful"),String::from("sleepy"),String::from("happy"),String::from("grumpy"),String::from("doc")].into_iter();
        let generator = CharacterChainGenerator::builder().train(inputs).build();
    }


}