use std::time::Duration;

use crate::{
    api::models::{IncomeRequest, IncomeResponse},
    model::{AccessToken, Check, RefreshToken, TokenNewError},
};
use log::debug;
use reqwest::{header::AUTHORIZATION, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use super::models::{DeviceInfo, TaxPayer, TokenRefreshRequest, TokenResponse};

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
    ) -> RequestResult<R> {
        self.request::<(), R>(Method::GET, api_method, None, access_token)
    }

    pub(super) fn post<B: Serialize, R: DeserializeOwned>(
        &self,
        api_method: &str,
        payload: Option<&B>,
        access_token: Option<&AccessToken>,
    ) -> RequestResult<R> {
        self.request(Method::POST, api_method, payload, access_token)
    }

    fn request<B: Serialize, R: DeserializeOwned>(
        &self,
        http_method: Method,
        api_method: &str,
        payload: Option<&B>,
        access_token: Option<&AccessToken>,
    ) -> RequestResult<R> {
        let url = Self::build_url(api_method);
        let mut req_builder = self.client.request(http_method.clone(), &url);

        if let Some(b) = payload {
            req_builder = req_builder.json(b)
        }

        if let Some(t) = access_token {
            req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", t.value.clone()))
        }

        debug!(
            "Запрос в АПИ: {} {} с токеном {:?}",
            http_method, url, access_token
        );
        let resp = req_builder.send()?;

        let status_code = resp.status();

        if status_code == StatusCode::UNAUTHORIZED {
            let text = resp.text()?;
            return Err(RequestError::Unauthorized(text));
        }

        if status_code != StatusCode::OK {
            let text = resp.text()?;
            return Err(RequestError::Not200(status_code, text));
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

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("send request")]
    SendRequest(#[from] reqwest::Error),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("got {0} status code: {1}")]
    Not200(StatusCode, String),

    #[error("decode response body")]
    DecodeResponseBody(#[from] serde_json::Error),

    #[error("create token")]
    Token(#[from] TokenNewError),
}

pub struct AuthorizedClient {
    client: InnerClient,
    device_id: String,
    inn: String,
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

impl AuthorizedClient {
    pub fn new(
        device_id: String,
        access_token: AccessToken,
        refresh_token: RefreshToken,
    ) -> anyhow::Result<Self> {
        let mut client = Self {
            client: InnerClient::new()?,
            device_id,
            inn: String::new(),
            access_token,
            refresh_token,
        };

        let taxpayer = client.taxpayer()?;

        client.inn = taxpayer.inn;

        Ok(client)
    }

    pub fn taxpayer(&mut self) -> RequestResult<TaxPayer> {
        self.get("/v1/taxpayer")
    }

    pub fn register_income(&mut self, check: Check) -> anyhow::Result<String> {
        let req = IncomeRequest::from(check);

        debug!("Request {:?}", req);
        let resp: IncomeResponse = self.post("/v1/income", Some(&req))?;

        let url = format!(
            "{}/{}/{}/print",
            BASE_URL, self.inn, resp.approved_receipt_uuid
        );

        Ok(url)
    }

    fn get<R: DeserializeOwned>(&mut self, api_method: &str) -> RequestResult<R> {
        match self.client.get(api_method, Some(&self.access_token)) {
            Ok(v) => Ok(v),
            Err(RequestError::Unauthorized(_)) => {
                self.refresh_token()?;
                self.client.get(api_method, Some(&self.access_token))
            }
            Err(e) => Err(e),
        }
    }

    fn post<B: Serialize, R: DeserializeOwned>(
        &mut self,
        api_method: &str,
        payload: Option<&B>,
    ) -> RequestResult<R> {
        match self
            .client
            .post(api_method, payload, Some(&self.access_token))
        {
            Ok(v) => Ok(v),
            Err(RequestError::Unauthorized(_)) => {
                self.refresh_token()?;
                self.client
                    .post(api_method, payload, Some(&self.access_token))
            }
            Err(e) => Err(e),
        }
    }

    fn refresh_token(&mut self) -> RequestResult<()> {
        let req = TokenRefreshRequest {
            device_info: DeviceInfo::new(&self.device_id),
            refresh_token: self.refresh_token.value.clone(),
        };

        let resp: TokenResponse = self.client.post("/v1/auth/token", Some(&req), None)?;

        self.access_token = AccessToken::new(resp.token, Some(resp.token_expires_in))?;
        self.refresh_token = RefreshToken::new(resp.refresh_token, resp.refresh_token_expires_in)?;

        Ok(())
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
