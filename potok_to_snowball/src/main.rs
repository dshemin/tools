use calamine::{
    deserialize_as_date_or_string, open_workbook, DataType, HeaderRow, RangeDeserializerBuilder,
    Reader, Xlsx,
};
use chrono::NaiveDate;
use clap::Parser;
use csv::Writer;
use log::info;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(bin_name = env!("CARGO_BIN_NAME"))]
enum Cli {
    #[command(about = "Prints tool version")]
    #[command(long_about = None)]
    Version,

    #[command(about = "Transform Potok report to Snowball format")]
    #[command(long_about = None)]
    Transform(TransformArgs),
}

#[derive(clap::Args)]
struct TransformArgs {
    #[arg(long)]
    in_path: PathBuf,

    #[arg(long)]
    out_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    match Cli::parse() {
        Cli::Version => {
            println!(env!("CARGO_PKG_VERSION"));
        }
        Cli::Transform(args) => {
            transform(args)?;
        }
    }

    Ok(())
}

/// Представление одной значащей записи в отчёте от Потока.
#[derive(Deserialize, Debug)]
struct PotokRecord {
    /// Дата совершённой операции.
    #[serde(rename = "Дата", deserialize_with = "deserialize_as_date_or_string")]
    date: Result<NaiveDate, String>,

    /*
        #[serde(rename = "Неделя")]
        week: i32,

        #[serde(rename = "Месяц")]
        month: i32,
    */
    /// Тип операции.
    #[serde(rename = "Тип")]
    r#type: String,

    /// Приход в рублях.
    #[serde(rename = "Приход (руб.)")]
    income: f32,

    /// Расход в рублях.
    #[serde(rename = "Расход (руб.)")]
    outcome: f32,

    /*
        #[serde(rename = "Комментарий")]
        comment: String,

        #[serde(rename = "Номер договора")]
        contract_number: i32,

        #[serde(rename = "ИНН")]
        inn: String,

        #[serde(rename = "Наименование/ФИО контрагента")]
        name: String,
    */
    // НДФЛ на совершённую операцию.
    #[serde(rename = "Размер удержанного НДФЛ")]
    fee: f32,
}

/// Трансформирует отчёт Потока в формат Snowball.
fn transform(args: TransformArgs) -> Result<(), Box<dyn std::error::Error>> {
    info!("Используем отчёт {:?}", args.in_path);

    let mut src: Xlsx<_> = open_workbook(args.in_path)?;
    let mut dest = Writer::from_path(args.out_path)?;

    // Первых 4 строки - пустые, 5 строка содержит не значащий заголовок.
    let range = src
        .with_header_row(HeaderRow::Row(5))
        .worksheet_range("История операций")?;

    let size = range.get_size();
    let height = size.0;

    // Крайняя строка в отчёте от Потока содержит итог и он нам не интересен.
    // На всякий случай убедимся что это так.
    let last_idx = {
        let mut idx = height - 1;
        if let Some(v) = range.get((height - 1, 0)) {
            if v.is_empty() {
                idx -= 1;
            }
        }
        idx
    };

    let iter =
        RangeDeserializerBuilder::with_deserialize_headers::<PotokRecord>().from_range(&range)?;

    for result in iter.enumerate() {
        if result.0 >= last_idx {
            break;
        }
        let in_record: PotokRecord = result.1?;

        let out_record: SnowballRecord = in_record.try_into()?;

        dest.serialize(out_record)?;
    }

    dest.flush()?;

    Ok(())
}

/// Представление одной записи в отчёте для Snowball.
#[derive(Serialize, Debug, PartialEq)]
struct SnowballRecord {
    #[serde(rename = "Event")]
    event: SnowballRecordEvent,

    #[serde(rename = "Date")]
    date: String,

    #[serde(rename = "Symbol")]
    symbol: String,

    #[serde(rename = "Price")]
    price: f32,

    #[serde(rename = "Quantity")]
    quantity: f32,

    #[serde(rename = "Currency")]
    currency: String,

    #[serde(rename = "FeeTax")]
    fee_tax: f32,

    #[serde(rename = "Exchange")]
    exchange: Option<String>,

    #[serde(rename = "NKD")]
    nkd: Option<f32>,

    #[serde(rename = "FeeCurrency")]
    fee_currency: Option<f32>,

    #[serde(rename = "DoNotAdjustCash")]
    do_not_adjust_cash: Option<String>,

    #[serde(rename = "Note")]
    note: Option<String>,
}

impl TryFrom<PotokRecord> for SnowballRecord {
    type Error = String;

    fn try_from(value: PotokRecord) -> Result<Self, Self::Error> {
        let event: SnowballRecordEvent = value.r#type.try_into()?;
        Ok(SnowballRecord {
            event,
            date: match value.date {
                Ok(d) => d.format("%Y-%m-%d").to_string(),
                Err(e) => return Err(e),
            },
            symbol: match event {
                SnowballRecordEvent::CashIn | SnowballRecordEvent::CashGain => "RUB".to_string(),
                SnowballRecordEvent::Buy
                | SnowballRecordEvent::Sell
                | SnowballRecordEvent::Dividend => "ZAIMI_POTOK".to_string(),
            },
            price: match event {
                SnowballRecordEvent::Buy
                | SnowballRecordEvent::Sell
                | SnowballRecordEvent::CashIn
                | SnowballRecordEvent::CashGain => 1.0,
                SnowballRecordEvent::Dividend => 0.0,
            },
            quantity: match event {
                SnowballRecordEvent::Buy => value.outcome,
                SnowballRecordEvent::Dividend
                | SnowballRecordEvent::Sell
                | SnowballRecordEvent::CashIn
                | SnowballRecordEvent::CashGain => value.income,
            },
            currency: "RUB".to_string(),
            fee_tax: value.fee,
            exchange: match event {
                SnowballRecordEvent::CashIn | SnowballRecordEvent::CashGain => None,
                SnowballRecordEvent::Buy
                | SnowballRecordEvent::Sell
                | SnowballRecordEvent::Dividend => Some("CUSTOM_HOLDING".to_string()),
            },
            nkd: None,
            fee_currency: None,
            do_not_adjust_cash: None,
            note: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SnowballRecordEvent {
    /// Покупка актива.
    Buy,

    /// Продажа актива.
    Sell,

    /// Дивидент от актива.
    Dividend,

    /// Пополнение.
    CashIn,

    /// Получение средств на счёт от платформы, например получение % на остаток,
    /// акции и т.п.
    CashGain,
}

impl Serialize for SnowballRecordEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl TryFrom<String> for SnowballRecordEvent {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let ev = match value.as_ref() {
            "Выдача займа" => SnowballRecordEvent::Buy,
            "Возврат основного долга" => SnowballRecordEvent::Sell,
            "Получение дохода (проценты, пени)" => {
                SnowballRecordEvent::Dividend
            }
            "Пополнение л/с" => SnowballRecordEvent::CashIn,
            "Начисление % на остаток" => SnowballRecordEvent::CashGain,
            _ => return Err(format!("unknown {}", value)),
        };

        Ok(ev)
    }
}

impl std::fmt::Display for SnowballRecordEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Buy => "Buy",
            Self::Sell => "Sell",
            Self::Dividend => "Dividend",
            Self::CashIn => "Cash_In",
            Self::CashGain => "Cash_Gain",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod potok_record {
        use super::*;

        macro_rules! try_from_tests_success {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, input) = $value;
                    assert_eq!(expected, input.try_into().unwrap());
                }
            )*
            }
        }

        try_from_tests_success! {
            try_from_buy: (SnowballRecord{
                event: SnowballRecordEvent::Buy,
                date: "2025-01-02".to_string(),
                symbol: "ZAIMI_POTOK".to_string(),
                price: 1.0,
                quantity: 10.1,
                currency: "RUB".to_string(),
                fee_tax: 20.2,
                exchange: Some("CUSTOM_HOLDING".to_string()),
                nkd: None,
                fee_currency: None,
                do_not_adjust_cash: None,
                note: None,
            }, PotokRecord{
                date: Ok(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                r#type: "Выдача займа".to_string(),
                income: 0.0,
                outcome: 10.1,
                fee: 20.2,
            }),

            try_from_sell: (SnowballRecord{
                event: SnowballRecordEvent::Sell,
                date: "2025-01-02".to_string(),
                symbol: "ZAIMI_POTOK".to_string(),
                price: 1.0,
                quantity: 10.1,
                currency: "RUB".to_string(),
                fee_tax: 20.2,
                exchange: Some("CUSTOM_HOLDING".to_string()),
                nkd: None,
                fee_currency: None,
                do_not_adjust_cash: None,
                note: None,
            }, PotokRecord{
                date: Ok(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                r#type: "Возврат основного долга".to_string(),
                income: 10.1,
                outcome: 0.0,
                fee: 20.2,
            }),

            try_from_dividend: (SnowballRecord{
                event: SnowballRecordEvent::Dividend,
                date: "2025-01-02".to_string(),
                symbol: "ZAIMI_POTOK".to_string(),
                price: 0.0,
                quantity: 10.1,
                currency: "RUB".to_string(),
                fee_tax: 20.2,
                exchange: Some("CUSTOM_HOLDING".to_string()),
                nkd: None,
                fee_currency: None,
                do_not_adjust_cash: None,
                note: None,
            }, PotokRecord{
                date: Ok(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                r#type: "Получение дохода (проценты, пени)".to_string(),
                income: 10.1,
                outcome: 0.0,
                fee: 20.2,
            }),

            try_from_cash_in: (SnowballRecord{
                event: SnowballRecordEvent::CashIn,
                date: "2025-01-02".to_string(),
                symbol: "RUB".to_string(),
                price: 1.0,
                quantity: 10.1,
                currency: "RUB".to_string(),
                fee_tax: 20.2,
                exchange: None,
                nkd: None,
                fee_currency: None,
                do_not_adjust_cash: None,
                note: None,
            }, PotokRecord{
                date: Ok(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                r#type: "Пополнение л/с".to_string(),
                income: 10.1,
                outcome: 0.0,
                fee: 20.2,
            }),

            try_from_cash_gain: (SnowballRecord{
                event: SnowballRecordEvent::CashGain,
                date: "2025-01-02".to_string(),
                symbol: "RUB".to_string(),
                price: 1.0,
                quantity: 10.1,
                currency: "RUB".to_string(),
                fee_tax: 20.2,
                exchange: None,
                nkd: None,
                fee_currency: None,
                do_not_adjust_cash: None,
                note: None,
            }, PotokRecord{
                date: Ok(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                r#type: "Начисление % на остаток".to_string(),
                income: 10.1,
                outcome: 0.0,
                fee: 20.2,
            }),
        }
    }

    #[cfg(test)]
    mod snowball_record_event {
        use super::*;

        macro_rules! try_from_tests_success {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, input) = $value;
                    assert_eq!(expected, input.to_string().try_into().unwrap());
                }
            )*
            }
        }

        try_from_tests_success! {
            try_from_buy: (SnowballRecordEvent::Buy, "Выдача займа"),
            try_from_sell: (SnowballRecordEvent::Sell, "Возврат основного долга"),
            try_from_dividend: (SnowballRecordEvent::Dividend, "Получение дохода (проценты, пени)"),
            try_from_cash_in: (SnowballRecordEvent::CashIn, "Пополнение л/с"),
            try_from_cash_gain: (SnowballRecordEvent::CashGain, "Начисление % на остаток"),
        }

        #[test]
        fn try_from_fail() {
            let actual: Result<SnowballRecordEvent, String> = "foo".to_string().try_into();

            assert_eq!("unknown foo", actual.err().unwrap());
        }

        macro_rules! to_string_tests {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (expected, input) = $value;
                    assert_eq!(expected, input.to_string());
                }
            )*
            }
        }

        to_string_tests! {
            to_string_buy: ("Buy", SnowballRecordEvent::Buy),
            to_string_sell: ("Sell", SnowballRecordEvent::Sell),
            to_string_divident: ("Dividend", SnowballRecordEvent::Dividend),
            to_string_cash_in: ("Cash_In", SnowballRecordEvent::CashIn),
            to_string_cash_gain: ("Cash_Gain", SnowballRecordEvent::CashGain),
        }
    }
}
