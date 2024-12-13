use std::fmt::Display;

use bigdecimal::BigDecimal;
use fake::Fake;
use namada_sdk::token::{
    Amount as NamadaAmount, DenominatedAmount as NamadaDenominatedAmount,
    Denomination as NamadaDenomination,
};

use crate::id::Id;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Amount(NamadaAmount);

impl From<NamadaAmount> for Amount {
    fn from(amount: NamadaAmount) -> Amount {
        Amount(amount)
    }
}

impl From<BigDecimal> for Amount {
    fn from(amount: BigDecimal) -> Amount {
        Amount(
            NamadaAmount::from_string_precise(&amount.to_string())
                .expect("Invalid amount"),
        )
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Amount {
    pub fn as_i64(&self) -> i64 {
        let s = self.0.to_string();
        s.parse::<i64>()
            .expect("Cannot convert NamadaAmount to i64")
    }

    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn fake() -> Self {
        Self(NamadaAmount::from_u64((0..10000000).fake::<u64>()))
    }
}

pub type Denomination = u8;

pub struct DenominatedAmount(NamadaDenominatedAmount);

impl DenominatedAmount {
    pub fn native(amount: Amount) -> DenominatedAmount {
        Self(NamadaDenominatedAmount::native(amount.0))
    }

    pub fn to_string_precise(&self) -> String {
        self.0.to_string_precise()
    }
}

impl From<NamadaDenominatedAmount> for DenominatedAmount {
    fn from(amount: NamadaDenominatedAmount) -> DenominatedAmount {
        DenominatedAmount(amount)
    }
}

impl From<(Amount, Denomination)> for DenominatedAmount {
    fn from((amount, denom): (Amount, Denomination)) -> DenominatedAmount {
        DenominatedAmount(NamadaDenominatedAmount::new(
            amount.0,
            NamadaDenomination(denom),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Id,
    pub token: Token,
    pub amount: Amount,
    pub height: u32,
}

pub type Balances = Vec<Balance>;

impl Balance {
    pub fn fake() -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");
        let token_address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            owner: Id::Account(address.to_string()),
            token: Token::Native(Id::Account(token_address.to_string())),
            amount: Amount::fake(),
            height: (0..10000).fake::<u32>(),
        }
    }

    pub fn fake_with_token(token: Token) -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            owner: Id::Account(address.to_string()),
            token,
            amount: Amount::fake(),
            height: 0,
        }
    }
}
