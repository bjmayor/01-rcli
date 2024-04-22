use std::fs;

use clap::Parser;
use rcli::{
    process_csv, process_decode, process_encode, process_generate_key, process_genpass,
    process_http_serve, process_text_sign, process_text_verify, Opts, SubCommand,
};
use zxcvbn::zxcvbn;
// rcli csv -i input.csv -o output.json --header -d ','

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPass(opts) => {
            let password = process_genpass(
                opts.length,
                opts.uppercase,
                opts.lowercase,
                opts.numbers,
                opts.symbols,
            )?;
            println!("{}", password);
            // output the password strength in stderr
            let estimate = zxcvbn(&password, &[])?;
            eprintln!("Password strength: {}", estimate.score());
        }
        SubCommand::Base64(subcmd) => match subcmd {
            rcli::Base64SubCommand::Encode(opts) => {
                let encode = process_encode(&opts.input, opts.format)?;
                println!("{}", encode);
            }
            rcli::Base64SubCommand::Decode(opts) => {
                let decode = process_decode(&opts.input, opts.format)?;
                println!("{}", decode);
            }
        },
        SubCommand::Text(subcmd) => match subcmd {
            rcli::TextSubCommand::Sign(opts) => {
                let sig = process_text_sign(&opts.input, &opts.key, opts.format)?;
                println!("{}", sig);
            }
            rcli::TextSubCommand::Verify(opts) => {
                let verified = process_text_verify(&opts.input, &opts.key, opts.format, &opts.sig)?;
                println!("{}", verified);
            }
            rcli::TextSubCommand::Generate(opts) => {
                let keys = process_generate_key(opts.format)?;
                match opts.format {
                    rcli::TextSignFormat::Blake3 => {
                        let output = opts.output.join("blake3.txt");
                        fs::write(output, &keys[0])?;
                    }
                    rcli::TextSignFormat::Ed25519 => {
                        let dir = opts.output;
                        let output = dir.join("ed25519.sk");
                        fs::write(output, &keys[0])?;
                        let output = dir.join("ed25519.pk");
                        fs::write(output, &keys[1])?;
                    }
                }
            }
        },
        SubCommand::Http(subcmd) => match subcmd {
            rcli::HttpSubCommand::Serve(opts) => {
                process_http_serve(opts.dir, opts.port).await?;
            }
        },
    }
    Ok(())
}
