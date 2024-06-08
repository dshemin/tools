mod api;
mod cli;
mod config;
mod functions;
mod macros;
mod model;
mod state;
mod template;

use std::{error::Error, path::PathBuf};

use anyhow::anyhow;
use api::{AuthorizedClient, PhoneAuthenticator};
use clap::Parser;
use log::debug;
use model::{AccessToken, RefreshToken};
use state::State;
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
    config_path: PathBuf,

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
            debug!("Подгружаем конфиг из {:?}", args.config_path);
            let cfg = config::load(args.config_path)?;

            debug!("Подгружаем состояние из {:?}", cfg.state_path);
            let mut state = state::load(&cfg.state_path)?;

            let raw_tmpl = cfg
                .templates
                .get(&args.template)
                .ok_or(anyhow!("template {} not found", args.template))?
                .clone();

            let client = get_client(&state)?;

            debug!("Синхронизируем состояние с актуальными данными");
            state.access_token = Some(client.get_access_token());
            state.refresh_token = Some(client.get_refresh_token());
            state.taxpayer_identification_number = Some(client.get_inn());

            debug!("Сохраняем состояние в {:?}", cfg.state_path);
            state::save(&state, &cfg.state_path)?;

            let tmpl = compiled::Template::new(raw_tmpl)?;

            let values = cli::ask(tmpl.get_fields())?;

            let check = tmpl.build_check(&values)?;

            let url = client.register_income(check)?;

            println!("Чек доступен но URL: {}", url);

            cli_clipboard::set_contents(url)?;

            println!("Так же чек скопирован в буфер обмена");
        }
    };

    Ok(())
}

fn get_client(state: &State) -> anyhow::Result<api::AuthorizedClient> {
    // У нас есть несколько позитивных сценариев которые мы должны тут обработать:
    // 1. State пустой, аутентификация прошла успешно
    // 2. State не пустой, токен там валиден
    // 3. State не пустой, токен там не валиден
    //
    // Во всех иных случаях мы будем возвращать ошибку.

    // Для начала убедимся что токены есть и валидны.
    let mut access_token = state.access_token.clone();
    let mut refresh_token = state.refresh_token.clone();

    let refresh_token_expired = refresh_token
        .as_ref()
        .map(|t| t.is_expired())
        .unwrap_or_default();

    if access_token.is_none() || refresh_token_expired {
        // Аутентифицируемся снова.
        let (new_access_token, new_refresh_token) = authenticate(state)?;

        access_token = Some(new_access_token);
        refresh_token = Some(new_refresh_token);
    }

    // Пытаемся создать клиента. Логика рефреша в нём.
    // Тут можно спокойно делать unwrap так, как токены точно будут.
    AuthorizedClient::from_tokens(access_token.unwrap(), refresh_token.unwrap())
}

fn authenticate(state: &State) -> anyhow::Result<(AccessToken, RefreshToken)> {
    let phone = inquire::Text::new("Номер телефона").prompt()?;

    let auth = PhoneAuthenticator::new(phone.clone())?;

    let challenge_token = auth.challenge()?;

    let code = inquire::Text::new("Код верификации из СМС").prompt()?;

    auth.verify(state.device_id.clone(), challenge_token, code)
}
