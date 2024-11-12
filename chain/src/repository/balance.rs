use anyhow::Context;
use diesel::sql_types::BigInt;
use diesel::upsert::excluded;
use diesel::{
    sql_query, ExpressionMethods, PgConnection, QueryableByName, RunQueryDsl,
};
use orm::balances::BalancesInsertDb;
use orm::schema::{balances, ibc_token, token};
use orm::token::{IbcTokenInsertDb, TokenInsertDb};
use shared::balance::Balances;
use shared::token::Token;
pub const MAX_PARAM_SIZE: u16 = u16::MAX;

#[derive(QueryableByName)]
struct BalanceColCount {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

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

pub fn insert_balance_in_chunks(
    transaction_conn: &mut PgConnection,
    balances: Balances,
) -> anyhow::Result<()> {
    let balances_col_count = sql_query(
        "SELECT COUNT(*)
            FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = 'balances';",
    )
    .get_result::<BalanceColCount>(transaction_conn)?;

    for chunk in balances
        // We have to divide MAX_PARAM_SIZE by the number of columns in the
        // balances table to get the correct number of rows in the
        // chunk.
        .chunks((MAX_PARAM_SIZE as i64 / balances_col_count.count) as usize)
    {
        insert_balance(transaction_conn, chunk.to_vec())?
    }

    anyhow::Ok(())
}

pub fn insert_tokens(
    transaction_conn: &mut PgConnection,
    tokens: Vec<Token>,
) -> anyhow::Result<()> {
    let tokens_db = tokens.iter().map(TokenInsertDb::from).collect::<Vec<_>>();

    let ibc_tokens_db = tokens
        .iter()
        .filter_map(IbcTokenInsertDb::from_token)
        .collect::<Vec<_>>();

    // TODO: add tests for on conflict:
    // - inserting token with the same address twice should NOT throw an error
    // - inserting native token with different address should throw an error
    diesel::insert_into(token::table)
        .values(tokens_db)
        .on_conflict(token::columns::address)
        .do_nothing()
        .execute(transaction_conn)
        .context("Failed to update tokens in db")?;

    diesel::insert_into(ibc_token::table)
        .values(ibc_tokens_db)
        .on_conflict(ibc_token::columns::address)
        .do_nothing()
        .execute(transaction_conn)
        .context("Failed to update ibc tokens in db")?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {

    use anyhow::Context;
    use diesel::{BoolExpressionMethods, QueryDsl, SelectableHelper};
    use namada_sdk::token::Amount as NamadaAmount;
    use namada_sdk::uint::MAX_SIGNED_VALUE;
    use orm::balances::BalanceDb;
    use shared::balance::{Amount, Balance};
    use shared::id::Id;
    use test_helpers::db::TestDb;

    use super::*;

    /// Test that the function correctly handles an empty `balances` input.
    #[tokio::test]
    async fn test_insert_balance_with_empty_balances() {
        let db = TestDb::new();

        db.run_test(|conn| {
            insert_balance(conn, vec![])?;

            let queried_balance = query_all_balances(conn)?;

            assert_eq!(queried_balance.len(), 0,);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test the basic functionality of inserting a single balance.
    #[tokio::test]
    async fn test_insert_balance_with_single_balance() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let owner = Id::Account(
                "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz".to_string(),
            );
            let token = Token::Native(Id::Account(
                "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
            ));
            let amount = Amount::from(NamadaAmount::from_u64(100));

            let balance = Balance {
                owner: owner.clone(),
                token: token.clone(),
                amount: amount.clone(),
            };

            insert_tokens(conn, vec![token.clone()])?;

            insert_balance(conn, vec![balance.clone()])?;

            let queried_balance = query_balance_by_address(conn, owner, token)?;

            assert_eq!(Amount::from(queried_balance.raw_amount), amount);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the function updates existing balances when there is a
    /// conflict.
    #[tokio::test]
    async fn test_insert_balance_with_existing_balances_update() {
        let db = TestDb::new();

        let owner = Id::Account(
            "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz".to_string(),
        );
        let token = Token::Native(Id::Account(
            "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
        ));
        let amount = Amount::from(NamadaAmount::from_u64(100));

        let balance = Balance {
            owner: owner.clone(),
            token: token.clone(),
            amount: amount.clone(),
        };

        db.run_test(move |conn| {
            seed_balance(conn, vec![balance.clone()])?;

            let new_amount = Amount::from(NamadaAmount::from_u64(200));
            let new_balance = Balance {
                amount: new_amount.clone(),
                ..(balance.clone())
            };

            insert_balance(conn, vec![new_balance])?;

            let queried_balance =
                query_balance_by_address(conn, owner.clone(), token.clone())?;

            assert_eq!(Amount::from(queried_balance.raw_amount), new_amount);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test the function's behavior when inserting balances that cause a
    /// conflict.
    #[tokio::test]
    async fn test_insert_balance_with_conflicting_owners() {
        let db = TestDb::new();

        let owner = Id::Account(
            "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz".to_string(),
        );
        let token = Token::Native(Id::Account(
            "tnam1qxfj3sf6a0meahdu9t6znp05g8zx4dkjtgyn9gfu".to_string(),
        ));
        let amount = Amount::from(NamadaAmount::from_u64(100));

        let balance = Balance {
            owner: owner.clone(),
            token: token.clone(),
            amount: amount.clone(),
        };

        db.run_test(move |conn| {
            seed_balance(conn, vec![balance.clone()])?;

            let new_amount = Amount::from(NamadaAmount::from_u64(200));
            let new_balance = Balance {
                token: token.clone(),
                amount: new_amount.clone(),
                ..(balance.clone())
            };

            seed_tokens_from_balance(conn, vec![new_balance.clone()])?;

            insert_balance(conn, vec![new_balance])?;

            let queried_balance =
                query_balance_by_address(conn, owner.clone(), token.clone())?;

            assert_eq!(Amount::from(queried_balance.raw_amount), new_amount);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }
    /// Test the function's behavior when inserting balances that cause a
    /// conflict.
    #[tokio::test]
    async fn test_insert_balance_with_conflicting_tokens() {
        let db = TestDb::new();

        let owner = Id::Account(
            "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz".to_string(),
        );
        let token = Token::Native(Id::Account(
            "tnam1qxfj3sf6a0meahdu9t6znp05g8zx4dkjtgyn9gfu".to_string(),
        ));
        let amount = Amount::from(NamadaAmount::from_u64(100));

        let balance = Balance {
            owner: owner.clone(),
            token: token.clone(),
            amount: amount.clone(),
        };

        db.run_test(move |conn| {
            seed_balance(conn, vec![balance.clone()])?;

            let new_owner = Id::Account(
                "tnam1q9rhgyv3ydq0zu3whnftvllqnvhvhm270qxay5tn".to_string(),
            );
            let new_amount = Amount::from(NamadaAmount::from_u64(200));
            let new_balance = Balance {
                amount: new_amount.clone(),
                owner: new_owner.clone(),
                ..(balance.clone())
            };

            insert_balance(conn, vec![new_balance])?;

            let queried_balance =
                query_balance_by_address(conn, owner.clone(), token.clone())?;

            let queried_balance_new = query_balance_by_address(
                conn,
                new_owner.clone(),
                token.clone(),
            )?;

            assert_eq!(Amount::from(queried_balance.raw_amount), amount);
            assert_eq!(
                Amount::from(queried_balance_new.raw_amount),
                new_amount
            );

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test the function's ability to handle a large number of balance inserts
    /// efficiently.
    #[tokio::test]
    async fn test_insert_balance_with_large_number_of_balances() {
        let db = TestDb::new();

        db.run_test(move |conn| {
            let token = Token::Native(Id::Account(
                "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
            ));
            let fake_balances = (0..10000)
                .map(|_| Balance::fake_with_token(token.clone()))
                .collect::<Vec<_>>();

            seed_tokens_from_balance(conn, fake_balances.clone())?;

            insert_balance(conn, fake_balances.clone())?;

            assert_eq!(query_all_balances(conn)?.len(), fake_balances.len());

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test how the function handles extremely large balance values.
    #[tokio::test]
    async fn test_insert_balance_with_extremely_large_balance_value() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let owner = Id::Account(
                "tnam1qqshvryx9pngpk7mmzpzkjkm6klelgusuvmkc0uz".to_string(),
            );
            let token = Token::Native(Id::Account(
                "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
            ));
            let max_amount = Amount::from(NamadaAmount::from(MAX_SIGNED_VALUE));

            let balance = Balance {
                owner: owner.clone(),
                token: token.clone(),
                amount: max_amount.clone(),
            };

            insert_tokens(conn, vec![token.clone()])?;

            insert_balance(conn, vec![balance.clone()])?;

            let queried_balance = query_balance_by_address(conn, owner, token)?;

            assert_eq!(Amount::from(queried_balance.raw_amount), max_amount);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that we can insert more than u16::MAX balances
    #[tokio::test]
    async fn test_insert_balance_in_chunks_with_max_param_size_plus_one() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let mps = MAX_PARAM_SIZE as u32;

            let token = Token::Native(Id::Account(
                "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
            ));

            // We have to fake_with_token otherwise we won't be able to seed
            // MAX_PARAM_SIZE + 1 tokens and test will panic
            let balances = (0..mps + 1)
                .map(|_| Balance::fake_with_token(token.clone()))
                .collect::<Vec<_>>();

            insert_tokens(conn, vec![token])?;

            let res = insert_balance_in_chunks(conn, balances);

            assert!(res.is_ok());

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that we can insert less than u16::MAX balances using chunks
    #[tokio::test]
    async fn test_insert_balance_in_chunks_with_1000_params() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let token = Token::Native(Id::Account(
                "tnam1q87wtaqqtlwkw927gaff34hgda36huk0kgry692a".to_string(),
            ));
            let balances = (0..1000)
                .map(|_| Balance::fake_with_token(token.clone()))
                .collect::<Vec<_>>();

            insert_tokens(
                conn,
                balances
                    .iter()
                    .map(|balance| balance.token.clone())
                    .collect::<Vec<_>>(),
            )?;

            seed_tokens_from_balance(conn, balances.clone())?;

            let res = insert_balance_in_chunks(conn, balances);

            assert!(res.is_ok());

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    fn seed_tokens_from_balance(
        conn: &mut PgConnection,
        balance: Vec<Balance>,
    ) -> anyhow::Result<()> {
        insert_tokens(
            conn,
            balance
                .iter()
                .map(|balance| balance.token.clone())
                .collect::<Vec<_>>(),
        )?;

        anyhow::Ok(())
    }

    fn seed_balance(
        conn: &mut PgConnection,
        balances: Vec<Balance>,
    ) -> anyhow::Result<()> {
        seed_tokens_from_balance(conn, balances.clone())?;

        diesel::insert_into(balances::table)
            .values::<&Vec<BalancesInsertDb>>(
                &balances
                    .into_iter()
                    .map(BalancesInsertDb::from_balance)
                    .collect::<Vec<_>>(),
            )
            .execute(conn)
            .context("Failed to update balances in db")?;

        anyhow::Ok(())
    }

    fn query_balance_by_address(
        conn: &mut PgConnection,
        owner: Id,
        token: Token,
    ) -> anyhow::Result<BalanceDb> {
        let token = match token {
            Token::Native(token) => token.to_string(),
            Token::Ibc(token) => token.address.to_string(),
        };

        balances::table
            .filter(
                balances::dsl::owner
                    .eq(owner.to_string())
                    .and(balances::dsl::token.eq(token)),
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
