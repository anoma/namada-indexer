use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::blocks::BlockInsertDb;
use orm::schema::blocks;
use shared::block::Block;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

pub fn upsert_block(
    transaction_conn: &mut PgConnection,
    block: Block,
    tm_block_response: TendermintBlockResponse,
) -> anyhow::Result<()> {
    diesel::insert_into(blocks::table)
        .values::<&BlockInsertDb>(&BlockInsertDb::from((
            block,
            tm_block_response,
        )))
        .on_conflict(blocks::height)
        .do_update()
        .set((
            blocks::hash.eq(excluded(blocks::hash)),
            blocks::app_hash.eq(excluded(blocks::app_hash)),
            blocks::timestamp.eq(excluded(blocks::timestamp)),
            blocks::proposer.eq(excluded(blocks::proposer)),
            blocks::epoch.eq(excluded(blocks::epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to insert block in db")?;

    anyhow::Ok(())
}
