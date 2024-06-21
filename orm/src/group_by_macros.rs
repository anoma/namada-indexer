use crate::schema::{bonds, validators};
use diesel::allow_columns_to_appear_in_same_group_by_clause;

allow_columns_to_appear_in_same_group_by_clause!(
    bonds::address,
    validators::id,
    validators::namada_address,
    validators::voting_power,
    validators::max_commission,
    validators::commission,
    validators::name,
    validators::email,
    validators::website,
    validators::description,
    validators::discord_handle,
    validators::avatar,
    validators::state,
);
