use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("NotAuthorized: {reason:?}")]
    NotAuthorized { reason: String },

    #[error("NotFound: {reason:?}")]
    NotFound { reason: String },

    #[error("ValidationError: {reason:?}")]
    ValidationError { reason: String },

    #[error("Unexpected: {reason:?}")]
    Unexpected { reason: String },
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}
