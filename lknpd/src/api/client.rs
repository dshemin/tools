use std::time::Duration;

use reqwest::{header::AUTHORIZATION, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use log::debug;
use crate::{api::models::{IncomeRequest, IncomeResponse}, model::{AccessToken, Check, RefreshToken}};

use super::models::TaxPayer;

/// Внутренний клиент.
pub(super) struct InnerClient {
    client: reqwest::blocking::Client,
}

const BASE_URL: &str = "https://lknpd.nalog.ru";
pub(super) const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

impl InnerClient {
    /// Создаёт новый инстанс внутреннего клиента.
    pub(super) fn new() -> anyhow::Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self { client })
    }

    pub(super) fn get<R: DeserializeOwned>(
        &self,
        api_method: &str,
        access_token: Option<&AccessToken>,
    ) -> anyhow::Result<R> {
        self.request::<(), R>(Method::GET, api_method, None, access_token)
    }

    pub(super) fn post<B: Serialize, R: DeserializeOwned>(
        &self,
        api_method: &str,
        payload: Option<&B>,
        access_token: Option<&AccessToken>,
    ) -> anyhow::Result<R> {
        self.request(Method::POST, api_method, payload, access_token)
    }

    fn request<B: Serialize, R: DeserializeOwned>(
        &self,
        http_method: Method,
        api_method: &str,
        payload: Option<&B>,
        access_token: Option<&AccessToken>,
    ) -> anyhow::Result<R> {
        let url = Self::build_url(api_method);
        let mut req_builder = self.client.request(http_method.clone(), &url);

        if let Some(b) = payload {
            req_builder = req_builder.json(b)
        }

        if let Some(t) = access_token {
            req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", t.value.clone()))
        }

        debug!("Запрос в АПИ: {} {} с токеном {:?}", http_method, url, access_token);
        let resp = req_builder.send()?;

        if resp.status() != StatusCode::OK {
            let text = resp.text()?;
            return Err(anyhow::anyhow!("got not 200: {}", text));
        }

        let body = resp.text()?;

        debug!("Тело ответа на {}: {}", url, body);

        let data = serde_json::from_str(&body)?;

        Ok(data)
    }

    fn build_url(method: &str) -> String {
        format!("{}/api/{}", BASE_URL, method.trim_start_matches('/'))
    }
}

pub struct AuthorizedClient {
    client: InnerClient,
    inn: String,
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

impl AuthorizedClient {
    pub fn from_tokens(
        access_token: AccessToken,
        refresh_token: RefreshToken,
    ) -> anyhow::Result<Self> {
        let mut client = Self {
            client: InnerClient::new()?,
            inn: String::new(),
            access_token,
            refresh_token,
        };

        let taxpayer = client.taxpayer()?;

        // todo(dshemin): Добавить логику рефреша токена.

        client.inn = taxpayer.inn;

        Ok(client)
    }

    pub fn taxpayer(&self) -> anyhow::Result<TaxPayer> {
        self.client.get("/v1/taxpayer", Some(&self.access_token))
    }

    pub fn register_income(&self, check: Check) -> anyhow::Result<String> {
        let req = IncomeRequest::from(check);

        debug!("Request {:?}", req);
        let resp: IncomeResponse = self.client.post("/v1/income", Some(&req), Some(&self.access_token))?;


        let url = format!("{}/{}/{}/print", BASE_URL, self.inn, resp.approved_receipt_uuid);

        Ok(url)
    }

    pub fn get_inn(&self) -> String {
        self.inn.clone()
    }

    pub fn get_access_token(&self) -> AccessToken {
        self.access_token.clone()
    }

    pub fn get_refresh_token(&self) -> RefreshToken {
        self.refresh_token.clone()
    }
}
