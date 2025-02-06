use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::gas::GasPriceDb;
use orm::parameters::ParametersInsertDb;
use orm::schema::{chain_parameters, gas_price};

pub fn upsert_chain_parameters(
    transaction_conn: &mut PgConnection,
    parameters: ParametersInsertDb,
) -> anyhow::Result<()> {
    diesel::insert_into(chain_parameters::table)
        .values(&parameters)
        .on_conflict(chain_parameters::chain_id)
        .do_update()
        .set((
            chain_parameters::apr.eq(excluded(chain_parameters::apr)),
            chain_parameters::max_block_time
                .eq(excluded(chain_parameters::max_block_time)),
            chain_parameters::cubic_slashing_window_length
                .eq(excluded(chain_parameters::cubic_slashing_window_length)),
            chain_parameters::checksums
                .eq(excluded(chain_parameters::checksums)),
        ))
        .execute(transaction_conn)
        .context("Failed to update chain_parameters state in db")?;

    Ok(())
}

pub fn upsert_gas_price(
    transaction_conn: &mut PgConnection,
    gas_price: Vec<GasPriceDb>,
) -> anyhow::Result<()> {
    diesel::insert_into(gas_price::table)
        .values(gas_price)
        .on_conflict(gas_price::token)
        .do_update()
        .set(gas_price::amount.eq(excluded(gas_price::amount)))
        .execute(transaction_conn)
        .context("Failed to update gas price in db")?;

    Ok(())
}
