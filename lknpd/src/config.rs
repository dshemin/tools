use std::{collections::HashMap, path::PathBuf};

use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};

use crate::template::raw::Template;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// ИИН самозанятого.
    pub inn: String,

    /// Данные для авторизации.
    pub auth: Auth,

    /// Список шаблонов.
    pub templates: HashMap<String, Template>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Auth {
    /// Токен авторизации.
    /// Можно получить на https://lknpd.nalog.ru/settings/public-access/
    pub token: String,
}

/// Загружает конфигурацию.
pub fn load(path: PathBuf) -> Result<Config, Error> {
    let cfg: Config = confy::load_path(path)?;

    Ok(cfg)
}
