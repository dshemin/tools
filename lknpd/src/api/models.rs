use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{api::client::USER_AGENT, model::{Check, Counterparty}};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxPayer {
    pub display_name: String,
    pub email: String,
    pub phone: String,
    pub inn: String,
    pub job_ids: Vec<String>,
    pub passport_series: String,
    pub passport_number: String,
    pub passport_issued_date: String,
    pub passport_issuer: String,
    pub spdul_code: String,
    pub birthday: String,
    pub address: String,
    // pub birthday_address: Option<String>,
    pub sex: String,
    pub avatar_exists: bool,
    pub registration_oktmo_code: String,
    pub registration_oktmo_name: String,
    pub oktmo: Oktmo,
    pub authority_name: String,
    pub authority_code: String,
    pub legal_registration_date: String,
    pub first_receipt_register_time: String,
    pub first_receipt_cancel_time: String,
    pub snils: String,
    pub pfr_info: PfrInfo,
    pub in_black_list: bool,
    // pub restrictions: Option<Vec<String>>,
    // pub temp_reg_date: Option<String>,
    pub special_tax_mode_info: SpecialTaxModeInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Oktmo {
    pub code: String,
    pub name: String,
    pub last_change_date: String,
    pub available_to_change_date: String,
    pub available_to_change: bool,
    // pub next_oktmo_code: Option<String>,
    // pub next_oktmo_name: Option<String>,
    // pub next_oktmo_from_code: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PfrInfo {
    // pub regnum: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialTaxModeInfo {
    // pub statuses: Option<Vec<String>>,
    // pub dead_line: Option<String>,
    // pub unreg_due_date: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SMSChallengeRequest {
    pub phone: String,

    pub required_tp_to_be_active: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SMSChallengeResponse {
    pub challenge_token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SMSVerifyRequest {
    pub challenge_token: String,
    pub device_info: DeviceInfo,
    pub code: String,
    pub phone: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub app_version: String,
    pub meta_details: DeviceInfoMetaDetails,
    pub source_device_id: String,
    pub source_type: String,
}

impl DeviceInfo {
    pub fn new(device_id: String) -> Self {
        Self {
            app_version: "1.0.0".to_owned(),
            meta_details: DeviceInfoMetaDetails {
                user_agent: USER_AGENT.to_owned(),
            },
            source_device_id: device_id.clone(),
            source_type: "WEB".to_owned(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoMetaDetails {
    pub user_agent: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SMSVerifyResponse {
    pub refresh_token: String,
    pub refresh_token_expires_in: Option<DateTime<Utc>>,
    pub token: String,
    #[serde(rename="tokenExpireIn")]
    pub token_expires_in: DateTime<Utc>,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeRequest {
    pub client: IncomeClientRequest,
    pub ignore_max_total_income_restriction: bool,
    pub operation_time: String,
    pub payment_type: String,
    pub request_time: String,
    pub services: Vec<IncomeServiceRequest>,
    pub total_amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeClientRequest {
    pub contact_phone: Option<String>,
    pub display_name: Option<String>,
    pub income_type: String,
    pub inn: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeServiceRequest {
    pub amount: u32,
    pub name: String,
    pub quantity: u32,
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

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeResponse {
    pub approved_receipt_uuid: String,
}
