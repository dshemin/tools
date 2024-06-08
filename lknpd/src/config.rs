use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::template::raw::Template;
use resolve_path::PathResolveExt;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// Путь до файла с состоянием.
    pub state_path: PathBuf,

    /// Список шаблонов.
    pub templates: HashMap<String, Template>,
}

/// Загружает конфигурацию.
pub fn load(path: PathBuf) -> anyhow::Result<Config> {
    let mut cfg: Config = confy::load_path(path)?;

    normalize(&mut cfg)?;

    Ok(cfg)
}

pub fn normalize(cfg: &mut Config) -> anyhow::Result<()> {
    // Чтобы правильно обработать относительные пути.
    cfg.state_path = cfg.state_path.try_resolve()?.into_owned();

    Ok(())
}
