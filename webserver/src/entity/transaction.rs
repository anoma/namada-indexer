use orm::{
    token::{IbcTokenDb, TokenDb},
    transactions::{TransactionResultDb, WrapperTransactionDb},
};
use shared::{
    id::Id,
    token::{IbcToken, Token},
};

#[derive(Debug, Clone)]
pub struct WrapperTransaction {
    pub id: Id,
    pub fee_payer: Id,
    pub fee_token: Token,
    pub gas_limit: String,
    pub gas_used: Option<u64>,
    pub amount_per_gas_unit: Option<String>,
    pub block_height: u64,
    pub exit_code: TransactionExitStatus,
    pub atomic: bool,
}

#[derive(Debug, Clone)]
pub enum TransactionExitStatus {
    Applied,
    Rejected,
}

impl WrapperTransaction {
    pub fn from_db(
        transaction: WrapperTransactionDb,
        tokens: Vec<(TokenDb, Option<IbcTokenDb>)>,
    ) -> Self {
        let fee_token = tokens
            .into_iter()
            .find_map(|(token, ibc_token)| {
                if transaction.fee_token == token.address {
                    match ibc_token {
                        Some(ibc_token) => Some(Token::Ibc(IbcToken {
                            address: Id::Account(ibc_token.address),
                            trace: Id::IbcTrace(ibc_token.ibc_trace),
                        })),
                        None => Some(Token::Native(Id::Account(token.address))),
                    }
                } else {
                    None
                }
            })
            .expect("Fee token should be known.");

        Self {
            id: Id::Hash(transaction.id),
            fee_payer: Id::Account(transaction.fee_payer),
            fee_token,
            gas_limit: transaction.gas_limit,
            gas_used: transaction.gas_used.map(|gas| gas as u64),
            amount_per_gas_unit: transaction.amount_per_gas_unit,
            block_height: transaction.block_height as u64,
            exit_code: match transaction.exit_code {
                TransactionResultDb::Applied => TransactionExitStatus::Applied,
                TransactionResultDb::Rejected => {
                    TransactionExitStatus::Rejected
                }
            },
            atomic: transaction.atomic,
        }
    }
}
