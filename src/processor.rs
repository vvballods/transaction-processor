use rust_decimal::Decimal;

use crate::{
    api::{ProcessingError, ProcessingResult},
    domain::{Account, StoredTransaction, TransactionId},
    state::StateStorage,
};

pub struct TransactionProcessor<S: StateStorage> {
    state: S,
}

impl<S: StateStorage> TransactionProcessor<S> {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub fn process(&self, transaction: StoredTransaction) -> ProcessingResult<()> {
        if transaction.is_not_valid() {
            tracing::error!("Transaction is not valid: {:?}", transaction);
            return Err(ProcessingError::TransactionIsNotValid {
                id: transaction.id().clone(),
            });
        }
        tracing::debug!("Processing: {:?}", transaction);
        let _ = self
            .state
            .insert_transaction(transaction.clone())
            .map(|tx| {
                let mut account = self.state.get_account(tx.client_id())?;
                if account.locked {
                    tracing::error!("Account is locked: {:?}", account);
                    return Err(ProcessingError::AccountIsLocked {
                        client_id: account.client,
                    });
                }
                self.adjust_account(&mut account, &tx)?;
                self.state.upsert_account(account)?;
                Ok(())
            })
            .map_err(|e| tracing::error!("Processing error {}", e));
        Ok(())
    }

    pub fn get_accounts(&self) -> ProcessingResult<Box<Vec<Account>>> {
        self.state.get_all_accounts()
    }

    fn adjust_account(
        &self,
        account: &mut Account,
        transaction: &StoredTransaction,
    ) -> ProcessingResult<()> {
        match transaction {
            StoredTransaction::Deposit { amount, .. } => self.deposit(account, amount),
            StoredTransaction::Withdrawal { amount, .. } => self.withdraw(account, amount),
            StoredTransaction::Dispute { id, .. } => self.dispute(account, id),
            StoredTransaction::Resolve { id, .. } => self.resolve(account, id),
            StoredTransaction::Chargeback { id, .. } => self.chargeback(account, id),
        }
    }

    fn deposit(&self, account: &mut Account, amount: &Decimal) -> ProcessingResult<()> {
        account.available += amount;
        account.total += amount;
        Ok(())
    }

    fn withdraw(&self, account: &mut Account, amount: &Decimal) -> ProcessingResult<()> {
        if account.available < amount.clone() {
            tracing::error!("Insufficient available funds in client's account");
            return Err(ProcessingError::AccountInsufficientAvailableFunds {
                client_id: account.client.clone(),
            });
        }
        account.available -= amount;
        account.total -= amount;
        Ok(())
    }

    fn dispute(&self, account: &mut Account, id: &TransactionId) -> ProcessingResult<()> {
        let tx = self.state.get_transaction(id.clone());
        match tx {
            Ok(tx) => {
                if let StoredTransaction::Deposit {
                    id,
                    client_id,
                    amount,
                    under_dispute,
                } = tx
                {
                    if account.client != client_id {
                        tracing::error!("Transaction can't be accessed by client");
                        return Err(ProcessingError::TransactionAccessDenied { id, client_id });
                    }
                    if under_dispute {
                        tracing::error!("Transaction already under dispute");
                        return Err(ProcessingError::TransactionAlreadyUnderDispute { id });
                    }
                    if account.available < amount {
                        tracing::error!("Insufficient available funds in client's account");
                        return Err(ProcessingError::AccountInsufficientAvailableFunds {
                            client_id,
                        });
                    }
                    account.available -= amount;
                    account.held += amount;
                    self.state.under_dispute(id, true)?;
                    Ok(())
                } else {
                    tracing::error!("Transaction {} is not a deposit", tx.id());
                    Err(ProcessingError::TransactionIsNotDisputable {
                        id: tx.id().clone(),
                    })
                }
            }
            Err(ProcessingError::TransactionNotFound { id }) => {
                tracing::info!("Ignoring dispute for non existing transaction {}.", id);
                Ok(())
            }
            Err(e) => Err(ProcessingError::UnknownError(e.to_string())),
        }
    }

    fn resolve(&self, account: &mut Account, id: &TransactionId) -> ProcessingResult<()> {
        let tx = self.state.get_transaction(id.clone());
        match tx {
            Ok(tx) => {
                if let StoredTransaction::Deposit {
                    id,
                    client_id,
                    amount,
                    under_dispute,
                } = tx
                {
                    if account.client != client_id {
                        tracing::error!("Transaction can't be accessed by client");
                        return Err(ProcessingError::TransactionAccessDenied { id, client_id });
                    }
                    if !under_dispute {
                        tracing::error!("Transaction is not under dispute");
                        return Ok(());
                    }
                    if account.held < amount {
                        tracing::error!("Insufficient held funds in client's account");
                        return Err(ProcessingError::AccountInsufficientHeldFunds { client_id });
                    }
                    account.available += amount;
                    account.held -= amount;
                    self.state.under_dispute(id, false)?;
                    Ok(())
                } else {
                    tracing::error!("Transaction {} is not a deposit", tx.id());
                    Err(ProcessingError::TransactionIsNotDisputable {
                        id: tx.id().clone(),
                    })
                }
            }
            Err(ProcessingError::TransactionNotFound { id }) => {
                tracing::info!("Ignoring dispute for non existing transaction {}.", id);
                Ok(())
            }
            Err(e) => Err(ProcessingError::UnknownError(e.to_string())),
        }
    }

    fn chargeback(&self, account: &mut Account, id: &TransactionId) -> ProcessingResult<()> {
        let tx = self.state.get_transaction(id.clone());
        match tx {
            Ok(tx) => {
                if let StoredTransaction::Deposit {
                    id,
                    client_id,
                    amount,
                    under_dispute,
                } = tx
                {
                    if account.client != client_id {
                        tracing::error!("Transaction can't be accessed by client");
                        return Err(ProcessingError::TransactionAccessDenied { id, client_id });
                    }
                    if !under_dispute {
                        tracing::error!("Transaction is not under dispute");
                        return Ok(());
                    }
                    if account.held < amount {
                        tracing::error!("Insufficient held funds in client's account");
                        return Err(ProcessingError::AccountInsufficientAvailableFunds {
                            client_id,
                        });
                    }
                    account.held -= amount;
                    account.total -= amount;
                    account.locked = true;
                    self.state.under_dispute(id, false)?;
                    Ok(())
                } else {
                    tracing::error!("Transaction {} is not a deposit", tx.id());
                    Err(ProcessingError::TransactionIsNotDisputable {
                        id: tx.id().clone(),
                    })
                }
            }
            Err(ProcessingError::TransactionNotFound { id }) => {
                tracing::info!("Ignoring dispute for non existing transaction {}.", id);
                Ok(())
            }
            Err(e) => Err(ProcessingError::UnknownError(e.to_string())),
        }
    }
}
