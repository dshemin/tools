use chrono::DateTime;
use log::debug;
use regex::Regex;
use std::{collections::HashMap, str::FromStr};

use crate::{functions, model};
use anyhow::anyhow;

use super::raw;

pub type Fields = HashMap<FieldName, Field>;
pub type Values = HashMap<FieldName, FieldValues>;
pub type FieldValues = HashMap<String, String>;

/// Скомпилированный шаблон.
#[derive(Debug, Clone)]
pub struct Template {
    fields: Fields,
    raw: raw::Template,
}

/// Представление одно поля шаблона.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Список всех найденных плейсхолдеров в данном поле.
    pub placeholders: Vec<Placeholder>,
}

impl Field {
    fn new(str: &str) -> anyhow::Result<Self> {
        Ok(Self {
            placeholders: template::template(str)?,
        })
    }
}

/// Все поля шаблона.
#[derive(
    Debug, Clone, PartialEq, Eq, std::hash::Hash, derive_more::Display, enum_iterator::Sequence,
)]
pub enum FieldName {
    #[display(fmt = "title")]
    Title,
    #[display(fmt = "price")]
    Price,
    #[display(fmt = "date")]
    Date,
    #[display(fmt = "organization name")]
    CounterpartyOrganizationName,
    #[display(fmt = "organization inn")]
    CounterpartyOrganizationINN,
}

impl Template {
    /// Создаёт новый инстанс скомпилированного шаблона.
    pub fn new(raw: raw::Template) -> anyhow::Result<Self> {
        Ok(Self {
            fields: Self::parse(&raw)?,
            raw,
        })
    }

    fn parse(raw: &raw::Template) -> anyhow::Result<Fields> {
        let mut fields = Fields::with_capacity(4);

        debug!("Parse field {}", FieldName::Title);
        let f = Field::new(&raw.title).map_err(|e| e.context("title"))?;
        fields.insert(FieldName::Title, f);

        debug!("Parse field {}", FieldName::Price);
        let f = Field::new(&raw.price).map_err(|e| e.context("price"))?;
        fields.insert(FieldName::Price, f);

        debug!("Parse field {}", FieldName::Date);
        let f = Field::new(&raw.date).map_err(|e| e.context("date"))?;
        fields.insert(FieldName::Date, f);

        if let raw::Counterparty::Organization { name, inn } = &raw.counterparty {
            debug!("Parse field {}", FieldName::CounterpartyOrganizationName);
            let f =
                Field::new(name).map_err(|e| e.context("organization name"))?;
            fields.insert(FieldName::CounterpartyOrganizationName, f);

            debug!("Parse field {}", FieldName::CounterpartyOrganizationINN);
            let f = Field::new(inn).map_err(|e| e.context("organization inn"))?;
            fields.insert(FieldName::CounterpartyOrganizationINN, f);
        }

        Ok(fields)
    }

    /// Возвращает список всех полей в шаблоне.
    pub fn get_fields(&self) -> &Fields {
        &self.fields
    }

    /// Собираем чек на основании данных в шаблоне и значений для переменных,
    /// которые задал пользователь.
    pub fn build_check(&self, values: &Values) -> anyhow::Result<model::Check> {
        let title = self.replace_values(FieldName::Title, &self.raw.title, values)?;
        let price = self.replace_values(FieldName::Price, &self.raw.price, values)?;
        let date = self.replace_values(FieldName::Date, &self.raw.date, values)?;
        let counterparty = match &self.raw.counterparty {
            raw::Counterparty::Person => model::Counterparty::Person,
            raw::Counterparty::Organization { name, inn } => {
                model::Counterparty::Organization {
                    name: (self.replace_values(FieldName::CounterpartyOrganizationName, name, values)?).try_into()?,
                    inn: (self.replace_values(FieldName::CounterpartyOrganizationINN, inn, values)?).try_into()?,
                }
            }
        };

        Ok(model::Check {
            title: title.try_into()?,
            price: price.parse()?,
            date: DateTime::parse_from_rfc3339(&date)?,
            counterparty,
        })
    }

    fn replace_values(
        &self,
        name: FieldName,
        input: &str,
        values: &Values,
    ) -> anyhow::Result<String> {
        debug!("Replace values for {}", name);
        match self.fields.get(&name) {
            Some(f) => {
                let mut result = String::from_str(input)?;

                if f.placeholders.is_empty() {
                    return Ok(result);
                }

                let values = values.get(&name).ok_or(anyhow!("there are no values"))?;

                for ph in f.placeholders.iter() {
                    let pattern = Self::create_pattern(ph);
                    let r = Regex::new(&pattern)?;
                    let val = Self::get_value(ph, values)?;
                    result = r.replace_all(&result, &val).to_string();
                }
                Ok(result)
            }
            None => Err(anyhow!("unhandled field {}", name)),
        }
    }

    fn create_pattern(ph: &Placeholder) -> String {
        let name = ph.name();
        format!(r"\{{\{{\s*{}.*?\}}\}}", name)
    }

    fn get_value(ph: &Placeholder, values: &FieldValues) -> anyhow::Result<String> {
        match ph {
            Placeholder::Variable { name, default: _ } => {
                values
                .get(name)
                .ok_or(anyhow!("field {} not found", name))
                .cloned()
            }
            Placeholder::Function { name } => {
                functions::execute(name).map_err(|e| anyhow!(e))
            }
        }


    }
}

enum Token {
    Skip,
    Placeholder(Placeholder),
}

/// Представление плейсхолдера в шаблоне.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placeholder {
    /// Переменная.
    /// Значение для неё будет затребовано у пользователя.
    Variable {
        /// Имя переменной.
        name: String,

        /// Значение по-умолчанию.
        default: Option<PlaceholderDefault>,
    },

    /// Функция.
    /// Результат будет вычислен и подставлен при формировании чека.
    Function {
        name: String,
    },
}

impl Placeholder {
    /// Возвращает название имя плейсхолдера.
    pub fn name(&self) -> String {
        match self {
            Self::Variable { name, default: _ } => name.clone(),
            Self::Function { name } => name.clone(),
        }
    }

    /// Возвращает значение по умолчанию.
    pub fn default(&self) -> Option<PlaceholderDefault> {
        match self {
            Self::Variable { name: _, default } => default.clone(),
            Self::Function { name: _ } => None,
        }
    }
}

/// Возможное значение по-умолчанию для плейсхолдера.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceholderDefault {
    /// Строка.
    /// Данная строка будет предложена как есть.
    String(String),

    /// Функция.
    /// Функция будет исполнена и результат исполнения будет предложен.
    Function(String),
}

// todo(dshemin): попробовать переписать парсер и убрать отсюда Token::Skip.

peg::parser! {
    grammar template() for str {
        pub rule template() -> Vec<Placeholder>
            = t:(placeholder() / skip())* {
                t.iter().filter_map(|x| {
                    match x {
                        Token::Placeholder(p) => Some((*p).clone()),
                        _ => None,
                    }
                }).collect()
            }

        rule placeholder() -> Token
            = r#"{{"# space()* p:(function() / variable()) space()* r#"}}"# {
                Token::Placeholder(p)
            }

        rule skip() -> Token
            = w:$((letter() / digit() / punctuation() / whitespace())+) {
                Token::Skip
            }

        rule variable() -> Placeholder
            = v:$(ident()) ":"? d:(variable_default())? {
                Placeholder::Variable{ name: v.to_owned(), default: d }
            }

        rule variable_default() -> PlaceholderDefault
            = variable_default_string() / variable_default_function()

        rule variable_default_string() -> PlaceholderDefault
            =  "\"" s:(string()) "\"" {
                PlaceholderDefault::String(s.to_owned())
            }

        rule variable_default_function() -> PlaceholderDefault
            = f:$(ident() space()*) function_arguments() {
                PlaceholderDefault::Function(f.to_owned())
            }

        rule function() -> Placeholder
            = f:$(ident() space()*) function_arguments() {
                Placeholder::Function{ name: f.to_owned() }
            }

        rule ident() -> String
            = i:$(en_letter() (en_letter() / digit())*) {? i.parse().or(Err("ident"))}

        rule string() -> String
            = s:$((letter() / digit() / space())+) {? s.parse().or(Err("string")) }

        rule function_arguments() = "()"

        rule digit() = ['0'..='9']

        rule letter() = en_letter() / ru_letter()

        rule en_letter() = ['a'..='z'] / ['A'..='Z']

        rule ru_letter() = ['а'..='я'] / ['А'..='Я'] / "ё" / "Ё"

        rule space() = " "

        rule whitespace() = space() / "\n" / "\r" / "\t"

        rule punctuation() = "," / "." / "?" / "!" / "-" / "'" / "\""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_pattern_for_variable() {
        let ph = Placeholder::Variable {
            name: "foo".to_owned(),
            default: Some(PlaceholderDefault::Function("bar".to_owned())),
        };

        let pattern = Template::create_pattern(&ph);
        assert_eq!(pattern, format!(r"\{{\{{\s*{}.*?\}}\}}", ph.name()))
    }

    #[test]
    fn create_pattern_for_function() {
        let ph = Placeholder::Function {
            name: "foo".to_owned(),
        };

        let pattern = Template::create_pattern(&ph);
        assert_eq!(pattern, format!(r"\{{\{{\s*{}.*?\}}\}}", ph.name()))
    }

    #[test]
    fn template() {
        assert_eq!(
            template::template(
                r#"
            some text with variable {{ var1 }},
            variable with default value {{ var2:"123" }},
            variable with default function {{ var3:foo() }}
            and function {{bar()}} end
            "#
            ),
            Ok(vec![
                Placeholder::Variable {
                    name: "var1".to_owned(),
                    default: None
                },
                Placeholder::Variable {
                    name: "var2".to_owned(),
                    default: Some(PlaceholderDefault::String("123".to_owned())),
                },
                Placeholder::Variable {
                    name: "var3".to_owned(),
                    default: Some(PlaceholderDefault::Function("foo".to_owned())),
                },
                Placeholder::Function {
                    name: "bar".to_owned()
                },
            ])
        );
    }
}
