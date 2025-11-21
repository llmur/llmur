use uuid::Uuid;
use crate::data::errors::AuthError;
use crate::data::password::Scheme;
use crate::data::utils::hash_content_2;

pub struct Scheme01;

impl Scheme for Scheme01 {
    fn hash(&self, content: &str, salt: &Uuid, application_secret: &Uuid) -> Result<String, AuthError> {
        hash_content_2(content, salt, application_secret).map_err(|e| AuthError::PasswordValidationFailed { cause: e.to_string() })
    }

    fn validate(&self, content: &str, reference: &str, salt: &Uuid, pepper: &Uuid) -> Result<(), AuthError> {
        let hashed = self.hash(content, salt, pepper)?;
        if hashed == reference { Ok(()) } else { Err(AuthError::InvalidPassword) }
    }
}