use diesel::data_types::PgNumeric;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;
use shared::balance::Amount;

pub struct Base10000BigUint(Option<BigUint>);

impl From<Option<BigUint>> for Base10000BigUint {
    fn from(value: Option<BigUint>) -> Self {
        Base10000BigUint(value)
    }
}

impl From<Amount> for Base10000BigUint {
    fn from(value: Amount) -> Self {
        let slice: Vec<u32> = value
            .0
            .iter()
            .map(|v| {
                let low: u32 = (v & 0xFFFFFFFF) as u32;
                let high: u32 = (v >> 32) as u32;
                vec![low, high]
            })
            .flatten()
            .collect();
        let big_uint = BigUint::from_slice(&slice);

        Base10000BigUint::from(Some(big_uint))
    }
}

impl Iterator for Base10000BigUint {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|v| {
            let (div, rem) = v.div_rem(&BigUint::from(10_000u16));
            if !div.is_zero() {
                self.0 = Some(div);
            }
            rem.to_i16().expect("remainder is at most 99_999")
        })
    }
}

pub struct PgNumericInt(PgNumeric);

impl PgNumericInt {
    pub fn into_inner(self) -> PgNumeric {
        self.0
    }
}

impl From<Base10000BigUint> for PgNumericInt {
    fn from(value: Base10000BigUint) -> Self {
        let mut base10000 = value.collect::<Vec<_>>();
        base10000.reverse();

        let unnecessary_zeroes =
            base10000.iter().rev().take_while(|i| i.is_zero()).count();

        let relevant_digits = base10000.len() - unnecessary_zeroes;
        let weight = base10000.len() as i16 - 1;
        base10000.truncate(relevant_digits);

        PgNumericInt(PgNumeric::Positive {
            weight,
            scale: 0,
            digits: base10000,
        })
    }
}
