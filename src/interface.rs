use rand::Rng;

pub trait RandomTextGenerator {
    fn generate_one<R: Rng + ?Sized>(&self, rng: &mut R) -> String;
}
