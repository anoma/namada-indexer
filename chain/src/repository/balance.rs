use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::balances::BalancesInsertDb;
use orm::schema::balances;
use shared::balance::Balances;

pub fn insert_balance(
    transaction_conn: &mut PgConnection,
    balances: Balances,
) -> anyhow::Result<()> {
    diesel::insert_into(balances::table)
        .values::<&Vec<BalancesInsertDb>>(
            &balances
                .into_iter()
                .map(BalancesInsertDb::from_balance)
                .collect::<Vec<_>>(),
        )
        .on_conflict((balances::columns::owner, balances::columns::token))
        .do_update()
        .set(
            balances::columns::raw_amount
                .eq(excluded(balances::columns::raw_amount)),
        )
        .execute(transaction_conn)
        .context("Failed to update balances in db")?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use std::convert::identity;

    use clap::Parser;
    use diesel::{BoolExpressionMethods, QueryDsl, SelectableHelper};
    use futures::FutureExt;
    use orm::balances::BalanceDb;
    use shared::{
        balance::{Amount, Balance},
        error::{AsDbError, ContextDbInteractError},
        id::Id,
    };

    use namada_sdk::token::Amount as NamadaAmount;

    use crate::{config::TestConfig, test_db::TestDb};

    use super::*;

    /// Test that the function correctly handles an empty `balances` input.
    #[tokio::test]
    async fn test_insert_balance_with_empty_balances() {
        let config = TestConfig::parse();
        let db = TestDb::new(&config);

        db.run_test(|pool| {
            async move {
                let conn = &mut pool.get().await.unwrap();

                conn.interact(move |conn| {
                    conn.build_transaction().read_write().run(|conn| {
                        insert_balance(conn, vec![])?;

                        let queried_balance = query_all_balances(conn)?;

                        assert_eq!(queried_balance.len(), 0,);

                        anyhow::Ok(())
                    })
                })
                .await
                .context_db_interact_error()
                .and_then(identity)
                .into_db_error()?;

                Ok(())
            }
            .boxed()
        })
        .await
        .expect("Failed to run test");
    }

    #[tokio::test]
    async fn test_insert_balance_with_single_balance() {
        let config = TestConfig::parse();
        let db = TestDb::new(&config);

        db.run_test(|pool| {
            async move {
                let conn = &mut pool.get().await.unwrap();

                conn.interact(move |conn| {
                    conn.build_transaction().read_write().run(|conn| {
                        let owner = Id::Account(
                            "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz"
                                .to_string(),
                        );
                        let token = Id::Account(
                            "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a"
                                .to_string(),
                        );
                        let amount = Amount::from(NamadaAmount::from_u64(100));

                        let balance = Balance {
                            owner: owner.clone(),
                            token: token.clone(),
                            amount: amount.clone(),
                        };
                        let balances = vec![balance.clone()];

                        insert_balance(conn, balances)?;

                        let queried_balance =
                            query_balance_by_address(conn, owner, token)?;

                        assert_eq!(
                            Amount::from(queried_balance.raw_amount),
                            amount
                        );

                        anyhow::Ok(())
                    })
                })
                .await
                .context_db_interact_error()
                .and_then(identity)
                .into_db_error()?;

                Ok(())
            }
            .boxed()
        })
        .await
        .expect("Failed to run test");
    }

    // /// Test that the function updates existing balances when there is a conflict.
    // #[test]
    // fn test_insert_balance_with_existing_balances_update() {
    //     // Implementation goes here
    // }

    // /// Test the function's behavior when inserting balances that cause a conflict.
    // #[test]
    // fn test_insert_balance_with_conflicting_balances() {
    //     // Implementation goes here
    // }

    // /// Test the function's error handling when the database connection is unavailable.
    // #[test]
    // fn test_insert_balance_when_database_is_unavailable() {
    //     // Implementation goes here
    // }

    // /// Test the function's ability to handle a large number of balance inserts efficiently.
    // #[test]
    // fn test_insert_balance_with_large_number_of_balances() {
    //     // Implementation goes here
    // }

    // /// Test how the function handles extremely large balance values.
    // #[test]
    // fn test_insert_balance_with_extremely_large_balance_value() {
    //     // Implementation goes here
    // }

    // /// Test the function's resilience against SQL injection attempts.
    // #[test]
    // fn test_insert_balance_with_sql_injection_attempt() {
    //     // Implementation goes here
    // }

    fn query_balance_by_address(
        conn: &mut PgConnection,
        owner: Id,
        token: Id,
    ) -> anyhow::Result<BalanceDb> {
        balances::table
            .filter(
                balances::dsl::owner
                    .eq(owner.to_string())
                    .and(balances::dsl::token.eq(token.to_string())),
            )
            .select(BalanceDb::as_select())
            .first(conn)
            .context("Failed to query balance by address")
    }

    fn query_all_balances(
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<BalanceDb>> {
        balances::table
            .select(BalanceDb::as_select())
            .get_results(conn)
            .context("Failed to query balance by address")
    }
}
