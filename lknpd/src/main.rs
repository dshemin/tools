mod cli;
mod client;
mod config;
mod functions;
mod macros;
mod model;
mod template;

use std::{error::Error, path::PathBuf};

use anyhow::anyhow;
use clap::Parser;
use template::compiled;

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(bin_name = env!("CARGO_BIN_NAME"))]
enum Cli {
    #[command(about = "Prints tool version")]
    #[command(long_about = None)]
    Version,

    #[command(about = "Make a check from provided template")]
    #[command(long_about = None)]
    Check(CheckArgs),
}

#[derive(clap::Args)]
struct CheckArgs {
    #[arg(short='c', long, default_value=Some("./config.toml"))]
    config_path: Option<PathBuf>,

    #[arg()]
    template: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    match Cli::parse() {
        Cli::Version => {
            println!(env!("CARGO_PKG_VERSION"));
        }
        Cli::Check(args) => {
            let cfg = config::load(args.config_path.unwrap())?;

            let raw_tmpl = cfg
                .templates
                .get(&args.template)
                .ok_or(anyhow!("template {} not found", args.template))?
                .clone();

            let tmpl = compiled::Template::new(raw_tmpl)?;

            let values = cli::ask(tmpl.get_fields())?;

            let check = tmpl.build_check(&values)?;

            let url = client::make_check(check, &cfg.auth.token, &cfg.inn)?;

            println!("Чек доступен но URL: {}", url);

            cli_clipboard::set_contents("url".to_owned())?;

            println!("Так же чек скопирован в буфер обмена");
        }
    };

    Ok(())
}
