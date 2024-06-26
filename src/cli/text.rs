use std::{fmt::Display, fs, path::PathBuf, str::FromStr};

use anyhow::Ok;
use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::{
    process_generate_key, process_text_decrypt, process_text_encrypt, process_text_sign,
    process_text_verify, CmdExector,
};

use super::{verify_file_exists, verify_path};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum TextSubCommand {
    #[command(about = "Sign text with a private/shared key and output a signature")]
    Sign(TextSignOpts),
    #[command(about = "Verify a signature against a public key")]
    Verify(TextVerifyOpts),
    #[command(about = "Generate a new key")]
    Generate(TextKeyGenOpts),
    #[command(about = "Encrypt text")]
    Encrypt(TextEncryptOpts),
    #[command(about = "Decrypt text")]
    Decrypt(TextDecryptOpts),
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long,value_parser=verify_file_exists,default_value="-")]
    pub input: String,
    #[arg(short, long,value_parser=verify_file_exists)]
    pub key: String,
    #[arg(long, default_value = "blake3", value_parser=parse_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short, long,value_parser=verify_file_exists,default_value="-" )]
    pub input: String,
    #[arg(short, long,value_parser=verify_file_exists)]
    pub key: String,
    #[arg(long, default_value = "blake3", value_parser=parse_format)]
    pub format: TextSignFormat,
    #[arg(short, long)]
    pub sig: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
}

fn parse_format(format: &str) -> Result<TextSignFormat, anyhow::Error> {
    format.parse()
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            _ => Err(anyhow::anyhow!("Invalid format: {}", s)),
        }
    }
}

impl From<TextSignFormat> for &'static str {
    fn from(format: TextSignFormat) -> Self {
        match format {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
        }
    }
}

impl Display for TextSignFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

#[derive(Debug, Parser)]
pub struct TextKeyGenOpts {
    #[arg(short, long, default_value = "blake3", value_parser=parse_format)]
    pub format: TextSignFormat,
    #[arg(short, long, value_parser=verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct TextEncryptOpts {
    #[arg(short, long,value_parser=verify_file_exists,default_value="-")]
    pub input: String,
    #[arg(short, long,value_parser=verify_file_exists)]
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct TextDecryptOpts {
    #[arg(short, long,value_parser=verify_file_exists,default_value="-" )]
    pub input: String,
    #[arg(short, long,value_parser=verify_file_exists)]
    pub key: String,
}

impl CmdExector for TextSignOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let sig = process_text_sign(&self.input, &self.key, self.format)?;
        println!("{}", sig);
        Ok(())
    }
}

impl CmdExector for TextVerifyOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let verified = process_text_verify(&self.input, &self.key, self.format, &self.sig)?;
        println!("{}", verified);
        Ok(())
    }
}

impl CmdExector for TextKeyGenOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let keys = process_generate_key(self.format)?;
        match self.format {
            TextSignFormat::Blake3 => {
                let output = self.output.join("blake3.txt");
                fs::write(output, &keys[0])?;
            }
            TextSignFormat::Ed25519 => {
                let dir = self.output.clone();
                let output = dir.join("ed25519.sk");
                fs::write(output, &keys[0])?;
                let output = dir.join("ed25519.pk");
                fs::write(output, &keys[1])?;
            }
        }
        Ok(())
    }
}

impl CmdExector for TextEncryptOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let encrypted = process_text_encrypt(&self.input, &self.key)?;
        println!("{}", encrypted);
        Ok(())
    }
}

impl CmdExector for TextDecryptOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let decrypted = process_text_decrypt(&self.input, &self.key)?;
        println!("{}", decrypted);
        Ok(())
    }
}
