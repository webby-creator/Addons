use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use rand_hc::Hc128Rng;
use time::OffsetDateTime;

thread_local! {
    static RNG: Hc128Rng = Hc128Rng::from_entropy();
}

pub fn gen_sample_alphanumeric<R: Rng>(amount: usize, rng: &mut R) -> String {
    rng.sample_iter(Alphanumeric)
        .take(amount)
        .map(char::from)
        .collect()
}

pub fn get_rng_secure() -> Hc128Rng {
    RNG.with(|v| v.clone())
}

/// 74 Characters Total. 64 Randomly generated. 10 are current unix time.
pub fn generate_file_name() -> String {
    intersperse_hash_with_time(gen_sample_alphanumeric(64, &mut get_rng_secure()))
}

/// 74 Characters Total. 64 Randomly generated. 4 are current year. 4 are random numbers. 2 dashes between.
pub fn generate_public_name() -> String {
    let mut rng = get_rng_secure();

    let name = gen_sample_alphanumeric(64, &mut rng);

    let year = OffsetDateTime::now_utc().year();
    let rand = rng.gen_range(0..=9999);

    format!("{year}-{rand:04}-{name}")
}

pub fn intersperse_hash_with_time(mut hash: String) -> String {
    let time = OffsetDateTime::now_utc().unix_timestamp().to_string();

    let mut time = time.chars().rev();

    for i in 0usize..10 {
        let pos = 60 - i * 6;

        hash.insert(pos, time.next().unwrap());
    }

    hash
}
