use anyhow::Result;
use chrono::Duration;
use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::{process_jwt_sign, process_jwt_verify, CmdExector};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum JwtSubCommand {
    #[command(name = "sign", about = "sign jwt")]
    Sign(JwtSignOpts),
    #[command(name = "verify", about = "verify jwt")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short, long)]
    pub sub: String,
    #[arg(short, long)]
    pub aud: String,
    #[arg(short, long, value_parser = parse_duration)]
    pub exp: Duration,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long)]
    pub token: String,
}

fn parse_duration(s: &str) -> Result<Duration> {
    let len = s.len();
    let (num_str, unit) = s.split_at(len - 1);
    let num = num_str.parse::<i64>()?;

    let duration = match unit {
        "d" => Duration::days(num),
        "w" => Duration::weeks(num),
        "m" => Duration::minutes(num),
        "h" => Duration::hours(num),
        _ => {
            return Err(anyhow::anyhow!("Invalid duration unit: {}", unit));
        }
    };

    Ok(duration)
}

impl CmdExector for JwtSignOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let token = process_jwt_sign(&self.sub, &self.aud, self.exp)?;
        println!("{}", token);
        Ok(())
    }
}

impl CmdExector for JwtVerifyOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let verified = process_jwt_verify(&self.token)?;
        println!("{:?}", verified);
        Ok(())
    }
}
