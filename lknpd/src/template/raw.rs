use serde::{Deserialize, Serialize};

/// Представление "сырого" шаблона.
/// В данном шаблоне могут присутствовать плейсхолдеры которые могу заменяться на
/// значения введённые пользователем или вычисленные из функций.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template {
    pub title: String,
    pub price: String,
    pub date: String,
    pub counterparty: Counterparty,
}

/// Представление "сырого" заказчика.
/// В полях данной структуры тоже могут присутствовать плейсхолдеры.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum Counterparty {
    #[default]
    Person,
    Organization {
        name: String,
        inn: String,
    },
}
