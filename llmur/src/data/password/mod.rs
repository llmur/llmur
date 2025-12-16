use crate::errors::{AsyncError, AuthenticationError, HashError};
use lazy_regex::regex_captures;
use std::hash::Hash;
use std::str::FromStr;
use uuid::Uuid;

mod scheme01;

// region:    --- Scheme
pub(crate) const DEFAULT_SCHEME: &str = "01";

#[derive(Debug)]
pub enum SchemeStatus {
    Ok,       // The pwd uses the latest scheme. All good.
    Outdated, // The pwd uses an old scheme.
}

pub(crate) trait Scheme {
    fn hash(&self, content: &str, salt: &Uuid, pepper: &Uuid) -> String;

    fn validate(
        &self,
        content: &str,
        reference: &str,
        salt: &Uuid,
        pepper: &Uuid,
    ) -> Result<(), AuthenticationError>;
}

pub(crate) enum SchemeDispatcher {
    Scheme01,
}

impl Scheme for SchemeDispatcher {
    fn hash(&self, content: &str, salt: &Uuid, pepper: &Uuid) -> String {
        match self {
            SchemeDispatcher::Scheme01 => scheme01::Scheme01.hash(content, salt, pepper),
        }
    }

    fn validate(
        &self,
        content: &str,
        reference: &str,
        salt: &Uuid,
        pepper: &Uuid,
    ) -> Result<(), AuthenticationError> {
        match self {
            SchemeDispatcher::Scheme01 => {
                scheme01::Scheme01.validate(content, reference, salt, pepper)
            }
        }
    }
}

pub(crate) fn get_scheme(scheme_name: &str) -> Result<impl Scheme, HashError> {
    match scheme_name {
        "01" => Ok(SchemeDispatcher::Scheme01),
        _ => Err(HashError::SchemeNotFound(scheme_name.to_string())),
    }
}
// endregion: --- Scheme

// region:    --- Password

pub async fn hash_password(
    password_clear: String,
    salt: Uuid,
    pepper: Uuid,
) -> Result<String, HashError> {
    tokio::task::spawn_blocking(move || {
        hash_for_scheme(DEFAULT_SCHEME, &password_clear, &salt, &pepper)
    })
    .await
    .map_err(|e| AsyncError::JoinError(e))?
}

pub async fn validate_password(
    password_clear: &str,
    reference: &str,
    salt: &Uuid,
    pepper: &Uuid,
) -> Result<SchemeStatus, AuthenticationError> {
    let PasswordParts {
        scheme_name,
        hashed,
    } = reference.parse()?;

    let scheme_status = if scheme_name == DEFAULT_SCHEME {
        SchemeStatus::Ok
    } else {
        SchemeStatus::Outdated
    };

    let password_clear = password_clear.to_string();
    let hashed = hashed.clone();
    let scheme_name = scheme_name.clone();
    let salt = *salt;
    let pepper = *pepper;

    // The closure captures only owned data with 'static lifetime
    tokio::task::spawn_blocking(move || {
        validate_for_scheme(&scheme_name, &password_clear, &hashed, &salt, &pepper)
    })
    .await
    .map_err(|e| AsyncError::JoinError(e))??;

    Ok(scheme_status)
}

fn hash_for_scheme(
    scheme_name: &str,
    to_hash: &str,
    salt: &Uuid,
    pepper: &Uuid,
) -> Result<String, HashError> {
    let hashed = get_scheme(scheme_name)?.hash(to_hash, salt, pepper);

    Ok(format!("#{scheme_name}#{hashed}"))
}

fn validate_for_scheme(
    scheme_name: &str,
    content: &str,
    reference: &str,
    salt: &Uuid,
    pepper: &Uuid,
) -> Result<(), AuthenticationError> {
    get_scheme(scheme_name)?.validate(content, reference, salt, pepper)
}

struct PasswordParts {
    /// The scheme only (e.g., "01")
    scheme_name: String,
    /// The hashed password,
    hashed: String,
}

impl FromStr for PasswordParts {
    type Err = AuthenticationError;

    fn from_str(pwd_with_scheme: &str) -> Result<Self, AuthenticationError> {
        regex_captures!(
            r#"^#(\w+)#(.*)"#, // Don't like to use regex but seems the simplest option
            pwd_with_scheme
        )
        .map(|(_, scheme, hashed)| Self {
            scheme_name: scheme.to_string(),
            hashed: hashed.to_string(),
        })
        .ok_or(AuthenticationError::PasswordSchemeParsingFailed)
    }
}
// endregion: --- Password
