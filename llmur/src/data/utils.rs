use uuid::Uuid;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::{Duration, Utc};
use hex::{decode, encode};
use rand::{Rng, distributions::Alphanumeric};
use sha2::{Digest, Sha256};

use crate::errors::{
    DbRecordConversionError, DecryptionError, EncryptionError, InvalidTimeFormatError,
};

pub trait ConvertInto<T>: Sized {
    // Required method
    fn convert(self, application_secret: &Option<Uuid>) -> Result<T, DbRecordConversionError>;
}

// region:    --- Time
pub fn parse_and_add_to_current_ts(input: &str) -> Result<i64, InvalidTimeFormatError> {
    // Check if the input ends with a valid unit and extract the numeric value
    if let Some((value_str, unit)) = input.split_at_checked(input.len() - 1) {
        let value: i64 = value_str.parse().map_err(|_| {
            InvalidTimeFormatError::TimeValueNotAValidNumber(
                input.to_string(),
                value_str.to_string(),
            )
        })?;

        // Determine the duration to add based on the unit
        let duration = match unit {
            "s" => Duration::seconds(value),
            "m" => Duration::minutes(value),
            "h" => Duration::hours(value),
            "d" => Duration::days(value),
            "w" => Duration::weeks(value),
            "M" => Duration::days(value * 30), // Approximation for 1 month as 30 days
            "y" => Duration::days(value * 365), // Approximation for 1 year as 365 days
            v => {
                return Err(InvalidTimeFormatError::InvalidTimePeriod(
                    input.to_string(),
                    v.to_string(),
                ));
            }
        };

        // Get the current UTC timestamp and add the duration
        let current_ts = current_timestamp_s();
        let new_ts = current_ts + duration.num_seconds();

        // Return the new timestamp as seconds since the UNIX epoch
        Ok(new_ts)
    } else {
        Err(InvalidTimeFormatError::InvalidTimeFormat(input.to_string()))
    }
}

pub fn current_timestamp_s() -> i64 {
    Utc::now().timestamp() // Returns the timestamp in seconds
}

pub fn current_timestamp_ms() -> i64 {
    Utc::now().timestamp_millis() // Returns the timestamp in seconds
}
// endregion: --- Time

// region:    --- Crypt

pub fn generate_random_api_key(key_suffix_length: usize) -> String {
    let api_key_rng: String = generate_random_alphanumeric_string(key_suffix_length);
    format!("sk-{}", api_key_rng)
}

pub fn generate_random_alphanumeric_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[tracing::instrument(level = "trace", name = "utils.string_to_uuid_v5", skip(input))]
pub fn new_uuid_v5_from_string(input: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, input.as_bytes())
}

#[tracing::instrument(level = "trace", name = "utils.encrypt", skip(input, salt, pepper))]
pub fn encrypt(input: &str, salt: &Uuid, pepper: &Uuid) -> Result<String, EncryptionError> {
    // Combine salt and pepper (if provided) as the key source
    let mut key_source = salt.to_string();
    key_source.push_str(&pepper.to_string());

    // Use SHA-256 to derive a 32-byte key from the salt+pepper string
    let key_hash = sha2::Sha256::digest(key_source.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_hash);

    // Generate a random nonce (12 bytes)
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Create the cipher instance
    let cipher = Aes256Gcm::new(key);

    // Encrypt the input string
    let ciphertext = cipher.encrypt(&nonce, input.as_bytes())?;

    // Encode nonce + ciphertext to base64 for easy storage/transmission
    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    let encrypted_base64 = encode(&encrypted_data);

    Ok(encrypted_base64)
}

#[tracing::instrument(
    level = "trace",
    name = "utils.decrypt",
    skip(encrypted_input, salt, pepper)
)]
pub fn decrypt(
    encrypted_input: &str,
    salt: &Uuid,
    pepper: &Uuid,
) -> Result<String, DecryptionError> {
    // Combine salt and pepper (if provided) as the key source
    let mut key_source = salt.to_string();
    key_source.push_str(&pepper.to_string());

    // Use SHA-256 to derive a 32-byte key from the salt+pepper string
    let key_hash = sha2::Sha256::digest(key_source.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_hash);

    // Decode the encrypted base64 string
    let encrypted_data = decode(encrypted_input)?;

    // Extract the nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&encrypted_data[..12]);

    // Extract the ciphertext (remaining bytes)
    let ciphertext = &encrypted_data[12..];

    // Create the cipher instance
    let cipher = Aes256Gcm::new(key);

    // Decrypt the ciphertext
    let decrypted_data = cipher.decrypt(nonce, ciphertext)?;

    // Convert the decrypted bytes back to a string
    let decrypted_string = String::from_utf8(decrypted_data)?;

    Ok(decrypted_string)
}

#[tracing::instrument(
    level = "trace",
    name = "utils.hash.v2",
    skip(content, salt, application_secret)
)]
pub fn hash_content_2(content: &str, salt: &Uuid, application_secret: &Uuid) -> String {
    let combined = format!("{}{}{}", salt, content, application_secret);

    // Create a SHA-256 hash of the combined string
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash = hasher.finalize();

    // Encode the hash as a hexadecimal string
    encode(hash)
}
// endregion: --- Crypt
