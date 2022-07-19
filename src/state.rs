use std::collections::HashMap;
use std::sync::RwLock;

use crate::{
    api::{ProcessingError, ProcessingResult},
    domain::{Account, ClientId, StoredTransaction, TransactionId},
};

pub trait StateStorage {
    fn get_transaction(&self, id: TransactionId) -> ProcessingResult<StoredTransaction>;
    fn insert_transaction(
        &self,
        transaction: StoredTransaction,
    ) -> ProcessingResult<StoredTransaction>;
    fn under_dispute(&self, id: TransactionId, under_dispute: bool) -> ProcessingResult<()>;

    fn get_all_accounts(&self) -> ProcessingResult<Box<Vec<Account>>>;
    fn get_account(&self, id: &ClientId) -> ProcessingResult<Account>;
    fn upsert_account(&self, account: Account) -> ProcessingResult<()>;
}

pub struct State {
    accounts: RwLock<HashMap<ClientId, Account>>,
    transactions: RwLock<HashMap<TransactionId, StoredTransaction>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
        }
    }
}

impl StateStorage for State {
    fn get_transaction(&self, id: TransactionId) -> ProcessingResult<StoredTransaction> {
        tracing::debug!("Retrieving all client account transactions");
        self.transactions
            .read()
            .map_err(|e| ProcessingError::UnknownError(e.to_string()))
            .and_then(|transactions| {
                transactions
                    .get(&id)
                    .cloned()
                    .ok_or(ProcessingError::TransactionNotFound { id })
            })
    }

    fn insert_transaction(
        &self,
        transaction: StoredTransaction,
    ) -> ProcessingResult<StoredTransaction> {
        match transaction {
            StoredTransaction::Deposit { .. } | StoredTransaction::Withdrawal { .. } => {
                tracing::debug!("Inserting: {:?}", transaction);
                self.transactions
                    .write()
                    .map_err(|e| ProcessingError::UnknownError(e.to_string()))
                    .and_then(|mut transactions| {
                        if !transactions.contains_key(&transaction.id()) {
                            transactions.insert(transaction.id().clone(), transaction.clone());
                            Ok(transaction)
                        } else {
                            Err(ProcessingError::TransactionAlreadyExists {
                                id: transaction.id().clone(),
                            })
                        }
                    })
            }
            _ => Ok(transaction),
        }
    }

    fn under_dispute(&self, id: TransactionId, under_dispute: bool) -> ProcessingResult<()> {
        tracing::debug!(
            "Updating transaction with id {} to under dispute = {}",
            id,
            under_dispute
        );
        self.transactions
            .write()
            .map_err(|e| ProcessingError::UnknownError(e.to_string()))
            .map(|mut transactions| {
                if let Some(tx) = transactions.get_mut(&id) {
                    tx.set_under_dispute(under_dispute);
                }
            })
    }

    fn get_all_accounts(&self) -> ProcessingResult<Box<Vec<Account>>> {
        tracing::debug!("Retrieving all client account balances");
        self.accounts
            .read()
            .map_err(|e| ProcessingError::UnknownError(e.to_string()))
            .map(|accounts| Box::new(accounts.values().cloned().collect::<Vec<_>>()))
    }

    fn get_account(&self, id: &ClientId) -> ProcessingResult<Account> {
        tracing::debug!("Retrieving account for client with id {} ", id);
        self.accounts
            .read()
            .map_err(|e| ProcessingError::UnknownError(e.to_string()))
            .map(|accounts| {
                accounts
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| Account::new(id.clone()))
            })
    }

    fn upsert_account(&self, account: Account) -> ProcessingResult<()> {
        tracing::debug!("Upserting {:?}", account);
        self.accounts
            .write()
            .map_err(|e| ProcessingError::UnknownError(e.to_string()))
            .map(|mut accounts| {
                accounts.insert(account.client, account.clone());
            })
    }
}
