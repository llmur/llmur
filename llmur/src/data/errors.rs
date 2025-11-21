#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    UtilsError(#[from] UtilsError),
    #[error(transparent)]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("FailedToGenerateSessionToken")]
    FailedToGenerateSessionToken,
    #[error("FailedToDecryptValue")]
    FailedToDecryptValue,
    #[error("InvalidDatabaseRecord")]
    InvalidDatabaseRecord,
    #[error("NothingToUpdate")]
    NothingToUpdate,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Failed to start Redis manager. Reason: ({cause})")]
    RedisStartError { cause: String },
    #[error("Failed to build redis pool. Reason: ({cause})")]
    RedisPoolBuilderError { cause: String },
    #[error("Error executing redis command. Reason: ({cause})")]
    RedisExecutionError { cause: String },
}

#[derive(thiserror::Error, Debug)]
pub enum DataConversionError {
    #[error("Convert Database Model: ({cause})")]
    DefaultError { cause: String },
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum UtilsError {
    #[error(transparent)]
    CryptUtilsError(#[from] CryptUtilsError),

    #[error("Invalid time time format")]
    InvalidTimeFormat
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum CryptUtilsError {
    #[error("Failed to encrypt")]
    EncryptError,
    #[error("Failed to decrypt string")]
    DecryptError
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Error validating password. Reason: ({cause})")]
    PasswordValidationFailed { cause: String },
    #[error("Invalid password scheme '{scheme}'")]
    SchemeNotFound { scheme: String },
    #[error("Error validating password. Reason: ({cause})")]
    PasswordHashFailed { cause: String },
    #[error("Failed to spawn thread for password validation. Reason: ({cause})")]
    SpawnThreadForValidationFailed { cause: String },
    #[error("Failed to parse hashed password")]
    PasswordParsingFailed
}