use namada_core::token::Amount;

pub fn raw_amount_to_nam(raw_amount: String) -> String {
    Amount::from_str(raw_amount, 0)
        .expect("raw_amount is not a valid string")
        .to_string_native()
}
