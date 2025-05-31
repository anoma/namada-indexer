use deadpool_diesel::postgres::Object;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use orm::cometbft::CometbftBlock;
use orm::schema::cometbft_block;
use shared::cometbft::CometbftBlock as CometBlock;

pub async fn get_block(
    conn: &Object,
    block_height: u32,
) -> anyhow::Result<Option<CometBlock>> {
    conn.interact(move |conn| {
        cometbft_block::table
            .find(block_height as i32)
            .select(CometbftBlock::as_select())
            .first(conn)
            .ok()
    })
    .await
    .map(|block| block.map(CometBlock::from))
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
