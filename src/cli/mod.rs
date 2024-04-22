mod base64;
mod csv;
mod genpass;
use std::path::{Path, PathBuf};
mod http;
mod text;

pub use base64::*;
use clap::Parser;
pub use csv::*;
pub use genpass::*;
pub use http::*;
pub use text::*;

#[derive(Debug, Parser)]
#[command(name = "rcli", version, about, author, long_about=None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV or Convert CSV to other formats")]
    Csv(CsvOpts),
    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
    #[command(subcommand)]
    Base64(Base64SubCommand),
    #[command(subcommand)]
    Text(TextSubCommand),
    #[command(subcommand)]
    Http(HttpSubCommand),
}

fn verify_file_exists(filename: &str) -> Result<String, String> {
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.to_string())
    } else {
        Err(format!("File not found: {}", filename))
    }
}
fn verify_path(path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err(format!("Path not found: {} or not a directory", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_file_exists() {
        assert_eq!(verify_file_exists("-"), Ok("-".to_string()));
        assert_eq!(
            verify_file_exists("Cargo.toml"),
            Ok("Cargo.toml".to_string())
        );
        assert_eq!(
            verify_file_exists("nonexistent"),
            Err("File not found: nonexistent".to_string())
        );
    }
}
