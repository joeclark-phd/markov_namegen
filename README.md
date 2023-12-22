# markov_namegen

This crate will define a couple of tools for generating random text (such as names) that will seem similar to a body of training data.  For example, you may train the model with a set of Roman names and you will get back a bunch of procedurally generated words that sound vaguely Roman-like, for example:

    postius
    rus
    gratus
    statlilius
    fricamian
    vitorialis
    barbanus
    civianus
    trifex
    majan

## Usage

To use, add `markov_namegen` to your `Cargo.toml`.

### RandomTextGenerator

One trait, RandomTextGenerator, is provided, with one method:

- `generate_one() -> String` yields a new, procedurally-generated text string.

There are three structs that implement the trait:

- CharacterChainGenerator
- CharacterChainCasePreservingGenerator (coming soon)
- ClusterChainGenerator

### CharacterChainGenerator

Quick start:

    use markov_namegen::CharacterChainGenerator;

    let dwarf_names = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();

    let generator = CharacterChainGenerator::builder()
        .train(dwarf_names)
        .build();

    println!(generator.generate_one());

(or using a file as input, and demonstrating all the builder options...)

    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use markov_namegen::CharacterChainGenerator;
    use markov_namegen::RandomTextGenerator;
    use rand::{rngs::SmallRng, SeedableRng};

    let file = File::open("resources/romans.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.unwrap());

    let namegen = CharacterChainGenerator::builder()
        .with_order(2)
        .with_prior(0.007)
        .with_pattern("^[A-Za-z]{4,8}$")
        .with_rng(Box::new(SmallRng::seed_from_u64(123)))
        .train(lines)
        .build();

    println!(generator.generate_one());

The big idea of Markov-chain random text generation is that you collect statistics on which characters follow other characters.  So if a particular language uses "th" a lot, "t" should often be followed by "h" in the randomly-generated text.  This crate's approach takes in an iterator of training data and uses it to build up a Markov model, which can be used to generate new strings. However, the Markov-chain approach has a number of caveats:

First, looking only at two-character sequences isn't very sophisticated. The model would be smarter if you looked back more than one letter.  For example, your model could know that "ot" and "nt" are often followed by "h" but "st" is not. The problem with that is that you will have far fewer examples of every 3-character, 4-character, or n-character sequences in your training data than you will have of 2-character sequences.  If a sequence never occurs in your training data, it can never occur in your output.  Because there are fewer examples, your output will be less random.

Based on an algorithm [described by JLund3 at RogueBasin](http://www.roguebasin.com/index.php/Names_from_a_high_order_Markov_Process_and_a_simplified_Katz_back-off_scheme),  which I have previously [implemented in Java](https://github.com/joeclark-phd/random-text-generators) and also [implemented in Python](https://github.com/joeclark-phd/roguestate/blob/master/program/namegen.py), CharacterChainGenerator mitigates these issues in a couple of ways:

- It develops models of multiple "orders", that is, of multiple lengths of character sequences.  If the generator encounters a new sequence of three characters like "rus", it will first check if it has trained a model on that sequence.  If not, it will fall back to check if it has a model for "us", failing that, it will certainly have a model for what comes after "s".  I call this a 3rd-order model, and it is the default.

- A Bayesian prior probability is added to every character in the alphabet in every model, so some truly random character sequences not seen in the training data are possible.  The alphabet is inferred from the training data, so any UTF-8 characters should be possible.  The default prior is a relative probability of 0.005.  Truly random output becomes more likely with a larger alphabet and with fewer trained character sequences, so you may want to play with this parameter: increase it to increase the randomness, or decrease it to make the output more like the training data.

Each newly generated candidate string is compared to the regex pattern provided (if any).  If the candidate string is filtered out, we generate another, until one passes. (Be aware that if you provide a very difficult-to-match pattern, generation time may increase greatly.  If you set up an impossible-to-match pattern, e.g. requiring characters that aren't in the training data set's alphabet, you will get an infinite loop.

CharacterChainGenerator ignores case, converting your input text and filters to lowercase and returning lowercase strings.

#### CharacterChainCasePreservingGenerator

(coming in a future version)

A variant of CharacterChainGenerator that learns and reproduces upper/lower case usage in the training data.  With a given dataset, this model may learn less effectively from the training data because it builds separate models for "A" and "a" (to give an example) instead of combining observations.  However, it may be preferable if the input data has interesting uses of capitalization (such as names that begin with "Mc" and "Mac" followed by capitals) that you want to re-generate.

### ClusterChainGenerator

Quick start:

    use markov_namegen::ClusterChainGenerator;
    use rand::{rngs::SmallRng, SeedableRng};

    let dwarf_names = vec!["dopey","sneezy","bashful","sleepy","happy","grumpy","doc"].into_iter();

    let namegen = ClusterChainGenerator::builder()
        .with_order(2)
        .without_prior()
        .with_pattern("^[A-Za-z]{4,8}$")
        .with_rng(Box::new(SmallRng::seed_from_u64(123)))
        .train(dwarf_names)
        .build();
    println!(generator.generate_one());


A class that uses a vowel/consonant clustering algorithm to generate new random text.  Based loosely on [an algorithm described by Kusigrosz at RogueBasin](http://www.roguebasin.com/index.php/Cluster_chaining_name_generator), it scans input text for clusters of vowels and clusters of consonants, after converting it all to lowercase, keeping track of all clusters that have been observed to follow any given cluster.  For example, "Elizabeth" would yield clusters `#-e-l-i-z-a-b-e-th-#` and "Anne" would yield `#-a-nn-e-#` where "`#`" is a control character marking the start or end of a string.

Much like CharacterChainGenerator, the implementation is based on a multi-order Markov chain. Internally we would keep track of the possible successors of each cluster, e.g.:

```
# -> [e,a]
e -> [l,th,#]
a -> [b,nn]
th -> [#]
...etc...
```

The `generateOne()` method takes a random walk through the cluster chain, only following paths that were found in the training data.  To continue our example, a new string could begin with "e" or "a", with equal likelihood, an "e" could be followed by "l", by "th", or by the end of a string, and so on.  With this training dataset of only two words, you could get a few different results, e.g.:

```
elizanneth
abelizanne
anneth
...etc...
```

## Release Notes

0.3.1: Added the ability to initialize each type of generator with a custom RNG.

0.2.1: Decreased default "prior" for ClusterChainGenerator.  A given training set will have way more clusters than letters in the alphabet, so the outcomes were way too random.

0.2.0: Added ClusterChainGenerator

0.1.3: CharacterChainGenerator working as intended