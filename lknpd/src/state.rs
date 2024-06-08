use std::{fs, io, path::Path};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::model::{AccessToken, RefreshToken};

/// Состояние приложения.
/// Хранит данные которые нужны между разными запусками приложения.
#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    /// Уникальный идентификатор устройства.
    pub device_id: String,

    /// Токен доступа к АПИ.
    pub access_token: Option<AccessToken>,

    /// Токен для обновления токена доступа.
    pub refresh_token: Option<RefreshToken>,

    /// ИНН самозанятого.
    pub taxpayer_identification_number: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let device_id = generate_device_id();

        Self {
            device_id,
            refresh_token: None,
            access_token: None,
            taxpayer_identification_number: None,
        }
    }
}

fn generate_device_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(21)
        .map(char::from)
        .collect()
}

/// Загружает состояние приложения из указанного файла.
pub fn load(path: &Path) -> LoadResult {
    if !path.exists() {
        return Ok(State::default());
    }

    let path = path.canonicalize()?;

    let content = fs::read_to_string(path)?;

    let state: State = serde_json::from_str(&content)?;

    Ok(state)
}

pub type LoadResult = std::result::Result<State, LoadError>;

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("read state file")]
    ReadFile(#[from] io::Error),

    #[error("deserialize")]
    Deserialize(#[from] serde_json::Error),
}

/// Сохраняет состояние приложения в указанный файл.
pub fn save(state: &State, path: &Path) -> SaveResult {
    let content = serde_json::to_string(state)?;

    fs::create_dir_all(path.parent().unwrap_or(Path::new("")))?;

    fs::write(path, content)?;

    Ok(())
}

pub type SaveResult = std::result::Result<(), SaveError>;

#[derive(thiserror::Error, Debug)]
pub enum SaveError {
    #[error("write state file")]
    WriteFile(#[from] io::Error),

    #[error("serialize")]
    Serialize(#[from] serde_json::Error),
}
