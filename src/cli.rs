use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pulse")]
#[command(version = "0.1.0")]
#[command(about = "Real-time homelab infrastructure monitor")]
pub struct Args {
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
}
