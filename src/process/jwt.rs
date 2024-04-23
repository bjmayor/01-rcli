use std::time::SystemTime;

use chrono::Duration;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
const JWTSECRET: &str = "rclijwtsecret";

pub fn process_jwt_sign(sub: &str, aud: &str, exp: Duration) -> anyhow::Result<String> {
    // get system current timestamp
    let now = SystemTime::now();
    // get the duration from the current time
    let exp = now
        .checked_add(std::time::Duration::from_secs(exp.num_seconds() as u64))
        .unwrap();
    // create a claim
    let claims = Claims {
        sub: sub.to_string(),
        company: aud.to_string(),
        exp: exp.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWTSECRET.as_ref()),
    )
    .unwrap();
    Ok(token)
}

pub fn process_jwt_verify(token: &str) -> anyhow::Result<bool> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWTSECRET.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(true)
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_jwt_sign_verify() {
        let sub = "acme";
        let aud = "device1";
        let exp = Duration::new(60, 0).unwrap();
        let token = process_jwt_sign(sub, aud, exp).unwrap();
        assert!(process_jwt_verify(token.as_str()).unwrap());
    }
}
