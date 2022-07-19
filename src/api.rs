use thiserror::Error;

use crate::domain::{ClientId, TransactionId};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProcessingError {
    #[error("Transaction with id {id} is not valid")]
    TransactionIsNotValid { id: TransactionId },
    #[error("Transaction with id {id} not found")]
    TransactionNotFound { id: TransactionId },
    #[error("Transaction with id {id} already exists")]
    TransactionAlreadyExists { id: TransactionId },
    #[error("Transaction with id {id} already under dispute")]
    TransactionAlreadyUnderDispute { id: TransactionId },
    #[error("Transaction with id {id} is not disputable")]
    TransactionIsNotDisputable { id: TransactionId },
    #[error("Transaction with id {id} can't be accessed by client with id {client_id}")]
    TransactionAccessDenied {
        id: TransactionId,
        client_id: ClientId,
    },
    #[error("Account with id {id} has insufficient available funds")]
    AccountInsufficientAvailableFunds { id: ClientId },
    #[error("Account with id {id} has insufficient held funds")]
    AccountInsufficientHeldFunds { id: ClientId },
    #[error("Account with id {id} is locked")]
    AccountIsLocked { id: ClientId },
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type ProcessingResult<T> = Result<T, ProcessingError>;
