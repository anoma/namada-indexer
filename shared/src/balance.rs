use std::fmt::Display;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use fake::Fake;
use namada_sdk::token::{
    Amount as NamadaAmount, DenominatedAmount as NamadaDenominatedAmount,
    Denomination as NamadaDenomination,
};
use num_bigint::BigUint;

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
        (&amount).into()
    }
}

impl From<&BigDecimal> for Amount {
    fn from(amount: &BigDecimal) -> Amount {
        let (big_int, _scale) = amount.as_bigint_and_scale();
        let (_sign, amount_bytes) = big_int.to_bytes_le();

        let uint_bytes: [u64; 4] = {
            // interpret as uint, regardless of sign. we
            // also truncate to the first 32 bytes.
            let mut uint_bytes = [0u8; 32];
            let min_len = amount_bytes.len().min(32);

            uint_bytes[..min_len].copy_from_slice(&amount_bytes[..min_len]);

            unsafe { std::mem::transmute(uint_bytes) }
        };

        Amount(NamadaAmount::from(namada_core::uint::Uint(uint_bytes)))
    }
}

impl From<Amount> for BigDecimal {
    fn from(amount: Amount) -> BigDecimal {
        let digits: [u8; 32] = {
            let uint: namada_core::uint::Uint = amount.0.into();
            unsafe { std::mem::transmute(uint.0) }
        };
        BigDecimal::from_biguint(BigUint::from_bytes_le(&digits), 0)
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

#[derive(Debug, Clone)]
pub struct TokenSupply {
    pub address: String,
    pub epoch: i32,
    pub total: BigDecimal,
    pub effective: Option<BigDecimal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_between_bigdec_and_amount() {
        let initial_amount =
            Amount(NamadaAmount::from(namada_core::uint::Uint([
                1u64, 2u64, 3u64, 4u64,
            ])));

        let initial_bigdec: BigDecimal =
            "25108406941546723056364004793593481054836439088298861789185"
                .parse()
                .unwrap();

        assert_eq!(initial_amount, Amount::from(&initial_bigdec));
        assert_eq!(BigDecimal::from(initial_amount.clone()), initial_bigdec);

        let amount_round_trip = {
            let x: BigDecimal = initial_amount.clone().into();
            let x: Amount = x.into();
            x
        };
        assert_eq!(initial_amount, amount_round_trip);

        let bigdec_round_trip = {
            let x: Amount = (&initial_bigdec).into();
            let x: BigDecimal = x.into();
            x
        };
        assert_eq!(initial_bigdec, bigdec_round_trip);
    }
}
