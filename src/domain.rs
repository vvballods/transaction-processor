use rust_decimal::Decimal;

pub type ClientId = u16;
pub type TransactionId = u32;
pub type Amount = Decimal;

const AMOUNT_PRECISION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoredTransaction {
    Deposit {
        id: TransactionId,
        client_id: ClientId,
        amount: Amount,
        under_dispute: bool,
    },
    Withdrawal {
        id: TransactionId,
        client_id: ClientId,
        amount: Amount,
    },
    Dispute {
        id: TransactionId,
        client_id: ClientId,
    },
    Resolve {
        id: TransactionId,
        client_id: ClientId,
    },
    Chargeback {
        id: TransactionId,
        client_id: ClientId,
    },
}

impl StoredTransaction {
    pub const fn id(&self) -> &TransactionId {
        match self {
            Self::Deposit { id, .. }
            | Self::Withdrawal { id, .. }
            | Self::Dispute { id, .. }
            | Self::Resolve { id, .. }
            | Self::Chargeback { id, .. } => id,
        }
    }

    pub const fn client_id(&self) -> &ClientId {
        match self {
            Self::Deposit { client_id, .. }
            | Self::Withdrawal { client_id, .. }
            | Self::Dispute { client_id, .. }
            | Self::Resolve { client_id, .. }
            | Self::Chargeback { client_id, .. } => client_id,
        }
    }

    pub fn is_not_valid(&self) -> bool {
        match self {
            Self::Deposit { amount, .. } => amount < &Amount::ZERO,
            Self::Withdrawal { amount, .. } => amount < &Amount::ZERO,
            _ => false,
        }
    }

    pub fn set_under_dispute(&mut self, is_under_dispute: bool) {
        if let StoredTransaction::Deposit {
            ref mut under_dispute,
            ..
        } = self
        {
            *under_dispute = is_under_dispute;
        }
    }
}

impl From<Transaction> for StoredTransaction {
    fn from(tx: Transaction) -> Self {
        match tx.transaction_type {
            TransactionType::Deposit => Self::Deposit {
                id: tx.tx,
                client_id: tx.client,
                amount: tx.amount.unwrap_or_default(),
                under_dispute: false,
            },
            TransactionType::Withdrawal => Self::Withdrawal {
                id: tx.tx,
                client_id: tx.client,
                amount: tx.amount.unwrap_or_default(),
            },
            TransactionType::Dispute => Self::Dispute {
                id: tx.tx,
                client_id: tx.client,
            },
            TransactionType::Resolve => Self::Resolve {
                id: tx.tx,
                client_id: tx.client,
            },
            TransactionType::Chargeback => Self::Chargeback {
                id: tx.tx,
                client_id: tx.client,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Account {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl Account {
    pub const fn new(client: ClientId) -> Self {
        Self {
            client,
            available: Amount::ZERO,
            held: Amount::ZERO,
            total: Amount::ZERO,
            locked: false,
        }
    }

    pub fn scaled(&mut self) {
        self.available = scale_to_amount_precision(self.available);
        self.held = scale_to_amount_precision(self.held);
        self.total = scale_to_amount_precision(self.total);
    }
}

fn scale_to_amount_precision(mut amount: Amount) -> Amount {
    if amount.scale() > AMOUNT_PRECISION {
        amount.rescale(AMOUNT_PRECISION);
    }
    amount
}
