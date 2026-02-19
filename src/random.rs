use rand::distr::{Alphanumeric, SampleString};

pub fn random_string(length: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, length)
}
