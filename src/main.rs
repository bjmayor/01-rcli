use clap::Parser;
use rcli::{CmdExector, Opts};

// rcli csv -i input.csv -o output.json --header -d ','

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    opts.cmd.execute().await?;
    Ok(())
}
