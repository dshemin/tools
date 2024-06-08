use enum_iterator::{all, cardinality};
use inquire::Text;
use inquire::validator::Validation;

use crate::{
    functions,
    template::compiled::{
        Field, FieldName, FieldValues, Fields, Placeholder,
        PlaceholderDefault, Values,
    },
};

/// Запрашивает пользовательский ввод для всех переменных, если они есть.
pub fn ask(fields: &Fields) -> anyhow::Result<Values> {
    let mut values = Values::with_capacity(cardinality::<FieldName>());

    for name in all::<FieldName>() {
        let res = ask_field(fields.get(&name))?;

        if let Some(fvs) = res {
            values.insert(name, fvs);
        }
    }

    Ok(values)
}

fn ask_field(fields: Option<&Field>) -> anyhow::Result<Option<FieldValues>> {
    fields.map_or(Ok(None), |f| {
        if f.placeholders.is_empty() {
            return Ok(None);
        }

        let mut values = FieldValues::with_capacity(f.placeholders.len());

        let vars = f
            .placeholders
            .iter()
            .filter(|ph| matches!(ph, Placeholder::Variable { name: _, default: _ }));

        for ph in vars {
            let value = prompt(ph)?;
            values.insert(ph.name(), value);
        }
        Ok(Some(values))
    })
}

fn prompt(ph: &Placeholder) -> anyhow::Result<String> {
    let title = format!("Значение для переменной \"{}\"", ph.name());

    let mut prompt = Text::new(&title);

    let default_value = compute_default_value(ph)?;

    if !default_value.is_empty() {
        prompt = prompt.with_default(&default_value);
    }

    // На данный момент считаем что все плейсхолдеры обязательны для заполнения.
    prompt = prompt.with_validator(|s: &str| {
        if s.is_empty() {
            return Ok(Validation::Invalid("required".into()))
        };
        Ok(Validation::Valid)
    });

    let val = prompt.prompt()?;

    Ok(val)
}

fn compute_default_value(ph: &Placeholder) -> anyhow::Result<String> {
    let val = match ph.default() {
        Some(d) => match d {
            PlaceholderDefault::String(s) => s,
            PlaceholderDefault::Function(n) => functions::execute(&n)?,
        },
        None => String::new(),
    };
    Ok(val)
}
