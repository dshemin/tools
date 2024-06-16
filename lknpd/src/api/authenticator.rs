use crate::model::{AccessToken, RefreshToken};

use super::{client::InnerClient, models::{DeviceInfo, SMSChallengeRequest, SMSChallengeResponse, SMSVerifyRequest, TokenResponse}};

/// Аутентификатор по номеру телефона.
///
/// Процесс аутентификации:
//  1. Запросить код верификации через https://lknpd.nalog.ru/api/v2/auth/challenge/sms/start
//  2. Код верификации передать на https://lknpd.nalog.ru/api/v1/auth/challenge/sms/verify
pub struct PhoneAuthenticator {
    client: InnerClient,
    phone: String,
}

impl PhoneAuthenticator {
    pub fn new(phone: String) -> anyhow::Result<Self> {
        Ok(Self {
            client: InnerClient::new()?,
            phone,
        })
    }

    /// Запрашивает проверочный код для указанного номера телефона.
    pub fn challenge(&self) -> anyhow::Result<String> {
        const URL: &str = "/v2/auth/challenge/sms/start";

        let payload = SMSChallengeRequest{
            phone: self.phone.clone(),
            required_tp_to_be_active: false,
        };

        let resp: SMSChallengeResponse = self.client.post(URL, Some(&payload), None)?;

        Ok(resp.challenge_token)
    }

    /// Обменивает проверочный код на токен авторизации.
    pub fn verify(
        &self,
        device_id: String,
        challenge_token: String,
        code: String,
    ) -> anyhow::Result<(AccessToken, RefreshToken)> {
        const URL: &str = "/v1/auth/challenge/sms/verify";

        let payload = SMSVerifyRequest {
            challenge_token,
            device_info: DeviceInfo::new(&device_id),
            code,
            phone: self.phone.clone(),
        };

        let resp: TokenResponse = self.client.post(URL, Some(&payload), None)?;

        let access_token = AccessToken::new(
            resp.token,
            Some(resp.token_expires_in),
        )?;

        let refresh_token = RefreshToken::new(
            resp.refresh_token,
            resp.refresh_token_expires_in,
        )?;

        Ok((access_token, refresh_token))
    }
}
