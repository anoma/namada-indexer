use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use orm::schema::{
    balances, bonds, governance_proposals, governance_votes, pos_rewards,
    unbonds, validators,
};

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("../orm/migrations/");

// TODO: what to do with this?
pub fn clear_db(conn: &mut PgConnection) -> anyhow::Result<()> {
    diesel::delete(balances::table)
        .execute(conn)
        .context("Failed to delete balances")?;

    diesel::delete(bonds::table)
        .execute(conn)
        .context("Failed to delete bonds")?;

    diesel::delete(governance_proposals::table)
        .execute(conn)
        .context("Failed to delete governance proposals")?;

    diesel::delete(governance_votes::table)
        .execute(conn)
        .context("Failed to delete governance votes")?;

    diesel::delete(pos_rewards::table)
        .execute(conn)
        .context("Failed to delete pos rewards")?;

    diesel::delete(unbonds::table)
        .execute(conn)
        .context("Failed to delete unbonds")?;

    diesel::delete(validators::table)
        .execute(conn)
        .context("Failed to delete validators")?;

    Ok(())
}

pub mod crawler_state;
pub mod pos;
