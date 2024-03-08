use crate::newtype;
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
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
