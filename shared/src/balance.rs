use std::fmt::Display;
use std::str::FromStr;

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

impl From<Amount> for BigDecimal {
    fn from(amount: Amount) -> BigDecimal {
        BigDecimal::from_str(&amount.0.to_string_native())
            .expect("Invalid amount")
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Amount {
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

#[cfg(test)]
mod tests {
    use crate::balance::{Amount, NamadaAmount};
    use bigdecimal::BigDecimal;
    use namada_sdk::token::NATIVE_MAX_DECIMAL_PLACES;
    use std::str::FromStr;
    #[test]
    fn test_bigquery_amount_round_trip_integer() {
        let bigdecimal =
            BigDecimal::from(100).with_scale(NATIVE_MAX_DECIMAL_PLACES.into());
        let amount = Amount::from(bigdecimal.clone());
        let bigdecimal_from_amount = BigDecimal::from(amount.clone());
        assert_eq!(amount, Amount::from(bigdecimal_from_amount.clone()));
        assert_eq!(bigdecimal, bigdecimal_from_amount);
    }
    #[test]
    fn test_bigquery_amount_round_trip_decimal() {
        let bigdecimal = BigDecimal::from_str("100.123456").unwrap();
        let amount = Amount::from(bigdecimal.clone());
        let bigdecimal_from_amount = BigDecimal::from(amount.clone());
        assert_eq!(amount, Amount::from(bigdecimal_from_amount.clone()));
        assert_eq!(bigdecimal, bigdecimal_from_amount);
    }
    #[test]
    fn test_amount_same_as_namada_amount_integer() {
        let amount = Amount::from(BigDecimal::from(100));
        let namada_amount = NamadaAmount::from_u64(100);
        assert_eq!(amount.0, namada_amount);
    }
    #[test]
    fn test_amount_same_as_namada_amount_decimal() {
        let amount = Amount::from(BigDecimal::from_str("100.123").unwrap());
        let namada_amount = NamadaAmount::from_string_precise("100.123")
            .expect("Invalid amount");
        assert_eq!(amount.0, namada_amount);
    }
}