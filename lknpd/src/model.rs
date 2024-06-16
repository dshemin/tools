use std::fmt::Display;

use crate::newtype;
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// Чек.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Check {
    /// Название оказанной услуги.
    pub title: Title,

    /// Цена оказанной услуги.
    pub price: Price,

    /// Дата выписки чека.
    pub date: DateTime<FixedOffset>,

    /// Тот кому мы оказали услугу.
    pub counterparty: Counterparty,
}

/// Представление всех возможных вариантов заказчиков.
#[derive(Serialize, Deserialize, Default, Debug)]
pub enum Counterparty {
    /// Физ. лицо.
    #[default]
    Person,

    /// Организация.
    Organization {
        name: OrganizationName,
        inn: OrganizationINN,
    },
}

newtype!(Title, String, "String", title_validate);

fn title_validate(value: &str) -> anyhow::Result<()> {
    if value.is_empty() {
        return Err(anyhow!("shouldn't be empty"));
    }

    Ok(())
}

newtype!(Price, u32, "u32");

newtype!(
    OrganizationName,
    String,
    "String",
    organization_name_validate
);

fn organization_name_validate(value: &str) -> anyhow::Result<()> {
    if value.len() < 3 {
        return Err(anyhow!("should be at least 3 chars"));
    }

    Ok(())
}

newtype!(OrganizationINN, String, "String", organization_inn_validate);

fn organization_inn_validate(value: &str) -> anyhow::Result<()> {
    if value.len() != 10 {
        return Err(anyhow!("should be exactly 10 digits"));
    }

    if !value.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("should contains only ascii digits"));
    }

    Ok(())
}

pub type AccessToken = Token;
pub type RefreshToken = Token;

/// Токент.
/// Может быть использован для того чтобы представляет как access, так и refresh
/// токен.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Token {
    /// Значение токена.
    pub(super) value: String,

    /// Дата и время когда токен станет не валидным.
    expire_at: Option<DateTime<Utc>>,
}

impl Token {
    /// Создаёт новый инстанс токена.
    pub fn new(value: String, expire_at: Option<DateTime<Utc>>) -> TokenNewResult {
        let value = value.trim().to_owned();
        if value.is_empty() {
            return Err(TokenNewError::EmptyValue);
        }

        let token = Self { value, expire_at };

        if token.is_expired() {
            return Err(TokenNewError::AlreadyExpired);
        }

        Ok(token)
    }

    /// Проверяет что токен уже протух.
    pub fn is_expired(&self) -> bool {
        matches!(self.expire_at, Some(date) if date < Utc::now())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub type TokenNewResult = std::result::Result<Token, TokenNewError>;

#[derive(Debug, thiserror::Error)]
pub enum TokenNewError {
    #[error("value is empty")]
    EmptyValue,

    #[error("already expired")]
    AlreadyExpired,
}
