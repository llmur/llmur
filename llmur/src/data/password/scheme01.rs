use uuid::Uuid;
use crate::data::password::Scheme;
use crate::data::utils::hash_content_2;
use crate::errors::AuthenticationError;

pub struct Scheme01;

impl Scheme for Scheme01 {
    fn hash(&self, content: &str, salt: &Uuid, application_secret: &Uuid) -> String {
        hash_content_2(content, salt, application_secret)
    }

    fn validate(&self, content: &str, reference: &str, salt: &Uuid, pepper: &Uuid) -> Result<(), AuthenticationError> {
        let hashed = self.hash(content, salt, pepper);
        if hashed == reference { Ok(()) } else { Err(AuthenticationError::InvalidPassword) }
    }
}