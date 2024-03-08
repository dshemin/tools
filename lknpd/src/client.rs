use log::debug;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::model::{Check, Counterparty};

pub fn make_check(check: Check, token: &str, inn: &str) -> anyhow::Result<String> {
    let client = get_client(token)?;

    let req = IncomeRequest::from(check);

    debug!("Request {:?}", req);

    let resp = client.post(build_url("income")).json(&req).send()?;

    if resp.status() != StatusCode::OK {
        let text = resp.text()?;
        return Err(anyhow::anyhow!("got not 200: {}", text));
    }

    let data: IncomeResponse = resp.json()?;

    debug!("Response {:?}", data);

    let url = build_url(&format!("{}/{}/print", inn, data.approved_receipt_uuid));

    Ok(url)
}

#[derive(Serialize, Debug)]
struct IncomeRequest {
    client: IncomeClientRequest,

    #[serde(rename = "ignoreMaxTotalIncomeRestriction")]
    ignore_max_total_income_restriction: bool,

    #[serde(rename = "operationTime")]
    operation_time: String,

    #[serde(rename = "paymentType")]
    payment_type: String,

    #[serde(rename = "requestTime")]
    request_time: String,

    services: Vec<IncomeServiceRequest>,

    #[serde(rename = "totalAmount")]
    total_amount: String,
}

#[derive(Serialize, Debug)]
struct IncomeClientRequest {
    #[serde(rename = "contactPhone")]
    contact_phone: Option<String>,

    #[serde(rename = "displayName")]
    display_name: Option<String>,

    #[serde(rename = "incomeype")]
    income_type: String,
    inn: Option<String>,
}

#[derive(Serialize, Debug)]
struct IncomeServiceRequest {
    amount: u32,
    name: String,
    quantity: u32,
}

impl From<Check> for IncomeRequest {
    fn from(value: Check) -> Self {
        let time = format!("{}", value.date.format("%FT%X%:z"));
        let price = value.price.into();

        Self {
            client: match value.counterparty {
                Counterparty::Person => IncomeClientRequest {
                    contact_phone: None,
                    display_name: None,
                    income_type: "FROM_INDIVIDUAL".into(),
                    inn: None,
                },
                Counterparty::Organization { name, inn } => IncomeClientRequest {
                    contact_phone: None,
                    display_name: Some(name.into()),
                    income_type: "FROM_LEGAL_ENTITY".into(),
                    inn: Some(inn.into()),
                },
            },
            ignore_max_total_income_restriction: false,
            operation_time: time.to_string(),
            payment_type: "CASH".to_string(),
            request_time: time.to_string(),
            services: vec![IncomeServiceRequest {
                amount: price,
                name: value.title.into(),
                quantity: 1,
            }],
            total_amount: format!("{}", price),
        }
    }
}

#[derive(Deserialize, Debug)]
struct IncomeResponse {
    #[serde(rename = "approvedReceiptUuid")]
    approved_receipt_uuid: String,
}

fn get_client(token: &str) -> anyhow::Result<reqwest::blocking::Client> {
    let authorization = format!("Bearer {}", token);
    let headers = {
        let mut m = HeaderMap::new();
        m.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&authorization)?,
        );
        m
    };

    let client = reqwest::blocking::ClientBuilder::new()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

#[inline]
fn build_url(method: &str) -> String {
    format!("https://lknpd.nalog.ru/api/v1/{}", method)
}
