use chrono::{Datelike, Local};

/// Исполняет запрошенную функцию и возвращает результат выполнения.
pub fn execute(name: &str) -> Result {
    let value = match name {
        "now" => Local::now().format("%FT%X%:z").to_string(),
        "year" => Local::now().year().to_string(),
        "month" => month(),
        _ => return Err(Error::UnknownFunction(name.to_owned())),
    };

    Ok(value)
}

fn month() -> String {
    match Local::now().month0() {
        0 => "Январь",
        1 => "Февраль",
        2 => "Март",
        3 => "Апрель",
        4 => "Май",
        5 => "Июнь",
        6 => "Июль",
        7 => "Август",
        8 => "Сентябрь",
        9 => "Октябрь",
        10 => "Ноябрь",
        11 => "Декабрь",
        _ => unreachable!("unknown month"),
    }
    .to_owned()
    .clone()
}

pub type Result = std::result::Result<String, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown function \"{0}\"")]
    UnknownFunction(String),
}
