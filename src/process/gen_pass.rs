use rand::seq::SliceRandom;
use rand::Rng;

use zxcvbn::zxcvbn;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const NUMBERS: &[u8] = b"123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*_";

pub fn process_genpass(
    length: u8,
    upper: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();
    if upper {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("UPPER won't be empty"));
    }
    if lowercase {
        chars.extend_from_slice(LOWERCASE);
        password.push(
            *LOWERCASE
                .choose(&mut rng)
                .expect("LOWERCASE won't be empty"),
        );
    }
    if numbers {
        chars.extend_from_slice(NUMBERS);
        password.push(*NUMBERS.choose(&mut rng).expect("NUMBERS won't be empty"));
    }
    if symbols {
        chars.extend_from_slice(SYMBOLS);
        password.push(*SYMBOLS.choose(&mut rng).expect("SYMBOLS won't be empty"));
    }
    for _ in 0..(length - password.len() as u8) {
        let idx = rng.gen_range(0..chars.len());
        password.push(chars[idx]);
    }
    // shuffle the password
    password.shuffle(&mut rng);

    let password = String::from_utf8(password)?;

    println!("{}", password);
    // output the password strength in stderr
    let estimate = zxcvbn(&password, &[])?;
    eprintln!("Password strength: {}", estimate.score());
    Ok(())
}
