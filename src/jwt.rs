use crate::settings::Settings;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

pub struct Jwt {
    header: Header,
}

impl Jwt {
    pub fn new() -> Self {
        let mut header = Header::new(Algorithm::HS256);
        header.cty = Some("twilio-fpa;v=1".to_owned());

        Jwt { header }
    }

    pub fn gen_token(&self, settings: &Settings, identity: String) -> String {
        let claims = Claims::new(settings, identity);
        let secret = settings.twilio.api_key_secret[..].as_bytes();
        encode(&self.header, &claims, &EncodingKey::from_secret(secret)).unwrap()
    }
}

type Timestamp = i64;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    jti: String,
    iss: String,    // Optional. Issuer
    sub: String,    // Optional. Subject (whom token refers to)
    iat: Timestamp, // Optional. Issued at (as UTC timestamp)
    exp: Timestamp, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    grants: ChatGrant,
}

impl Claims {
    fn new(settings: &Settings, identity: String) -> Self {
        let now = Utc::now();
        let rand_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Claims {
            jti: format!("{}-{}", settings.twilio.account_sid, rand_str),
            iss: format!("{}", settings.twilio.api_key_sid),
            sub: format!("{}", settings.twilio.account_sid),
            iat: now.timestamp(),
            exp: (now + Duration::hours(15)).timestamp(),
            grants: ChatGrant::new(identity),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatGrant {
    identity: String,
    chat: ChatGrantData,
}

impl ChatGrant {
    fn new(identity: String) -> Self {
        ChatGrant {
            identity,
            chat: ChatGrantData::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatGrantData {
    service_sid: String,
}

impl ChatGrantData {
    fn new() -> Self {
        ChatGrantData {
            service_sid: String::from("some_service_sid"),
        }
    }
}
