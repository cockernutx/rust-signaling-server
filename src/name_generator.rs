use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

macro_rules! read_names {
    ($file:expr $(,)?) => {
        include_str!($file).split_whitespace().collect()
    };
}

/// A naming strategy for the `Generator`
pub enum Name {
    /// This represents a plain naming strategy of the form `"ADJECTIVE-NOUN"`
    Plain,
    /// This represents a naming strategy with a random number appended to the
    /// end, of the form `"ADJECTIVE-NOUN-NUMBER"`
    Numbered,
}

impl Default for Name {
    fn default() -> Self {
        Name::Plain
    }
}

/// A random name generator which combines an adjective, a noun, and an
/// optional number
///
/// A `Generator` takes a slice of adjective and noun words strings and has
/// a naming strategy (with or without a number appended).
pub struct Generator<'a> {
    adjectives: Vec<&'a str>,
    nouns: Vec<&'a str>,
    naming: Name,
    rng: ThreadRng,
}

impl<'a> Generator<'a> {
    /// Constructs a new `Generator<'a>`
    ///
    /// # Examples
    ///
    /// ```
    /// use names::{Generator, Name};
    ///
    /// let adjectives = &["sassy"];
    /// let nouns = &["clocks"];
    /// let naming = Name::Plain;
    ///
    /// let mut generator = Generator::new(adjectives, nouns, naming);
    ///
    /// assert_eq!("sassy-clocks", generator.next().unwrap());
    /// ```
    pub fn new(adjectives: Vec<&'a str>, nouns: Vec<&'a str>, naming: Name) -> Self {
        Generator {
            adjectives,
            nouns,
            naming,
            rng: ThreadRng::default(),
        }
    }

    /// Construct and returns a default `Generator<'a>` containing a large
    /// collection of adjectives and nouns
    ///
    /// ```
    /// use names::{Generator, Name};
    ///
    /// let mut generator = Generator::with_naming(Name::Plain);
    ///
    /// println!("My new name is: {}", generator.next().unwrap());
    /// ```
    pub fn with_naming(naming: Name) -> Self {
        Generator::new(read_names!("../names/adjectives.txt"), read_names!("../names/nouns.txt"), naming)
    }
}

impl<'a> Default for Generator<'a> {
    fn default() -> Self {
        Generator::new(read_names!("../names/adjectives.txt"), read_names!("../names/nouns.txt"), Name::default())
    }
}

impl<'a> Iterator for Generator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let adj = self.adjectives.choose(&mut self.rng).unwrap();
        let noun = self.nouns.choose(&mut self.rng).unwrap();

        Some(match self.naming {
            Name::Plain => format!("{}-{}", adj, noun),
            Name::Numbered => format!("{}-{}-{:04}", adj, noun, rand_num(&mut self.rng)),
        })
    }
}

fn rand_num(rng: &mut ThreadRng) -> u16 {
    rng.gen_range(1..10000)
}