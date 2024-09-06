use std::collections::HashSet;

use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use orm::bond::BondInsertDb;
use orm::schema::{bonds, pos_rewards, unbonds, validators};
use orm::unbond::UnbondInsertDb;
use orm::validators::{ValidatorDb, ValidatorUpdateMetadataDb};
use shared::block::Epoch;
use shared::bond::Bonds;
use shared::id::Id;
use shared::unbond::{UnbondAddresses, Unbonds};
use shared::validator::ValidatorMetadataChange;

pub fn clear_bonds(
    transaction_conn: &mut PgConnection,
    addresses: Vec<(Id, Id)>,
) -> anyhow::Result<()> {
    // If there are no addresses to clear, return early.
    // Without this check, the query would delete all bonds in the table.
    if addresses.is_empty() {
        return Ok(());
    }

    let mut query = diesel::delete(bonds::table).into_boxed();

    for (source, validator) in addresses {
        query = query.or_filter(
            bonds::address.eq(source.to_string()).and(
                bonds::validator_id.eq_any(
                    validators::table.select(validators::columns::id).filter(
                        validators::columns::namada_address
                            .eq(validator.to_string()),
                    ),
                ),
            ),
        );
    }

    query
        .execute(transaction_conn)
        .context("Failed to remove bonds from db")?;

    anyhow::Ok(())
}

pub fn insert_bonds(
    transaction_conn: &mut PgConnection,
    bonds: Bonds,
) -> anyhow::Result<()> {
    diesel::insert_into(bonds::table)
        .values::<&Vec<BondInsertDb>>(
            &bonds
                .into_iter()
                .map(|bond| {
                    let validator: ValidatorDb = validators::table
                        // Epoch for validators is problematic?
                        .filter(
                            validators::namada_address
                                .eq(&bond.target.to_string()),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    BondInsertDb::from_bond(bond, validator.id)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            bonds::columns::validator_id,
            bonds::columns::address,
            bonds::columns::start,
        ))
        .do_update()
        .set((
            bonds::columns::raw_amount.eq(excluded(bonds::columns::raw_amount)),
        ))
        .execute(transaction_conn)
        .context("Failed to update bonds in db")?;

    anyhow::Ok(())
}

pub fn insert_unbonds(
    transaction_conn: &mut PgConnection,
    unbonds: Unbonds,
) -> anyhow::Result<()> {
    diesel::insert_into(unbonds::table)
        .values::<&Vec<UnbondInsertDb>>(
            &unbonds
                .into_iter()
                .map(|unbond| {
                    let validator: ValidatorDb = validators::table
                        .filter(
                            validators::namada_address
                                .eq(&unbond.target.to_string()),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    UnbondInsertDb::from_unbond(unbond, validator.id)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            unbonds::columns::validator_id,
            unbonds::columns::address,
            unbonds::columns::withdraw_epoch,
        ))
        .do_update()
        .set((
            unbonds::columns::raw_amount
                .eq(excluded(unbonds::columns::raw_amount)),
            // TODO: it's weird that we update the withdraw_epoch as it's a
            // part of on_conflict, it's most likely redundant
            unbonds::columns::withdraw_epoch
                .eq(excluded(unbonds::columns::withdraw_epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to update unbonds in db")?;
    anyhow::Ok(())
}

pub fn remove_withdraws(
    transaction_conn: &mut PgConnection,
    current_epoch: Epoch,
    unbond_addresses: HashSet<UnbondAddresses>,
) -> anyhow::Result<()> {
    let sources = unbond_addresses
        .iter()
        .map(|unbond| unbond.source.to_string())
        .collect::<Vec<String>>();

    let validators = unbond_addresses
        .iter()
        .map(|unbond| unbond.validator.to_string())
        .collect::<Vec<String>>();

    diesel::delete(
        unbonds::table.filter(
            unbonds::columns::address
                .eq_any(sources)
                .and(unbonds::columns::validator_id.eq_any(
                    validators::table.select(validators::columns::id).filter(
                        validators::columns::namada_address.eq_any(validators),
                    ),
                ))
                .and(unbonds::columns::withdraw_epoch.le(current_epoch as i32)),
        ),
    )
    .execute(transaction_conn)
    .context("Failed to remove withdraws from db")?;

    anyhow::Ok(())
}

pub fn delete_claimed_rewards(
    transaction_conn: &mut PgConnection,
    reward_claimers: HashSet<Id>,
) -> anyhow::Result<()> {
    diesel::delete(
        pos_rewards::table.filter(
            pos_rewards::dsl::owner
                .eq_any(reward_claimers.into_iter().map(|id| id.to_string())),
        ),
    )
    .execute(transaction_conn)
    .context("Failed to update reawrds in db")?;

    anyhow::Ok(())
}

pub fn update_validator_metadata(
    transaction_conn: &mut PgConnection,
    metadata_change: Vec<ValidatorMetadataChange>,
) -> anyhow::Result<()> {
    for metadata in metadata_change {
        let metadata_change_db = ValidatorUpdateMetadataDb {
            commission: metadata.commission,
            name: metadata.name,
            email: metadata.email,
            website: metadata.website,
            description: metadata.description,
            discord_handle: metadata.discord_handler,
            avatar: metadata.avatar,
        };
        diesel::update(validators::table)
            .set(metadata_change_db)
            .filter(validators::namada_address.eq(metadata.address.to_string()))
            .execute(transaction_conn)
            .context("Failed to update unbonds in db")?;
    }
    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use orm::bond::BondDb;
    use orm::unbond::UnbondDb;
    use orm::validators::ValidatorInsertDb;
    use shared::balance::Amount;
    use shared::bond::Bond;
    use shared::unbond::Unbond;
    use shared::validator::Validator;
    use test_helpers::db::TestDb;

    use super::*;

    /// Test that the function correctly handles an empty `addresses` input.
    #[tokio::test]
    async fn test_clear_bonds_with_empty_addresses() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let validator = Validator::fake();
            let bonds = (0..10)
                .map(|_| Bond::fake(validator.clone().address))
                .collect();

            seed_bonds(conn, validator, bonds)?;
            clear_bonds(conn, vec![])?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), 10);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the clear_bonds function does nothing when there are not bonds
    /// in the db.
    #[tokio::test]
    async fn test_clear_bonds_with_no_bonds() {
        let db = TestDb::new();

        db.run_test(|conn| {
            clear_bonds(conn, vec![])?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), 0);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the clear_bonds function removes the correct bonds from the
    /// db.
    #[tokio::test]
    async fn test_clear_bonds() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let validator = Validator::fake();
            let bonds: Vec<Bond> = (0..10)
                .map(|_| Bond::fake(validator.clone().address))
                .collect();

            seed_bonds(conn, validator.clone(), bonds.clone())?;

            let bonds_to_clear = bonds
                .iter()
                .take(5)
                .map(|bond| (bond.source.clone(), bond.target.clone()))
                .collect();

            clear_bonds(conn, bonds_to_clear)?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), 5);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the clear_bonds function correctly clears pairs when there are
    /// addresses duplicates
    #[tokio::test]
    async fn test_clear_bonds_with_duplicates() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let validator1 = Validator::fake();
            let validator2 = Validator::fake();
            let validator3 = Validator::fake();
            let bond1 = Bond::fake(validator1.clone().address);
            let bond2 = Bond::fake(validator2.clone().address);
            let bond3 = Bond::fake(validator2.clone().address);
            let bond4 = Bond::fake(validator3.clone().address);
            let bond5 = Bond::fake(validator3.clone().address);
            let bond6 = Bond::fake(validator1.clone().address);

            seed_bonds(
                conn,
                validator1.clone(),
                vec![bond1.clone(), bond6.clone()],
            )?;
            seed_bonds(
                conn,
                validator2.clone(),
                vec![bond2.clone(), bond3.clone()],
            )?;
            seed_bonds(
                conn,
                validator3.clone(),
                vec![bond4.clone(), bond5.clone()],
            )?;

            let bonds_to_clear = vec![
                (bond1.source, validator1.address),
                (bond3.source, validator2.address),
                (bond5.source, validator3.address),
            ];

            clear_bonds(conn, bonds_to_clear)?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), 3);
            // TODO: later compare whole bonds

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_bonds function correctly handles empty bonds input.
    #[tokio::test]
    async fn test_insert_bonds_with_empty_bonds() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_bonds: Vec<Bond> = (0..10)
                .map(|_| Bond::fake(fake_validator.clone().address))
                .collect();
            let fake_bonds_len = fake_bonds.len();
            seed_bonds(conn, fake_validator, fake_bonds)?;

            insert_bonds(conn, vec![])?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), fake_bonds_len);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_bonds function panics if validator is not in db.
    #[tokio::test]
    #[should_panic]
    async fn test_insert_bonds_with_missing_validator() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_bonds: Vec<Bond> = (0..10)
                .map(|_| Bond::fake(fake_validator.clone().address))
                .collect();

            insert_bonds(conn, fake_bonds)?;

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_bonds function correctly inserts bonds into the
    /// empty db.
    #[tokio::test]
    async fn test_insert_bonds_with_empty_db() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_bonds: Vec<Bond> = (0..10)
                .map(|_| Bond::fake(fake_validator.clone().address))
                .collect();
            let fake_bonds_len = fake_bonds.len();

            seed_validator(conn, fake_validator)?;

            insert_bonds(conn, fake_bonds)?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), fake_bonds_len);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_bonds function updates the raw_amount on conflict
    #[tokio::test]
    async fn test_insert_bonds_with_conflict() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_bonds_len = 10;
            let fake_bonds: Vec<Bond> = (0..fake_bonds_len)
                .map(|_| Bond::fake(fake_validator.clone().address))
                .collect();

            seed_bonds(conn, fake_validator.clone(), fake_bonds.clone())?;

            let mut updated_bonds = fake_bonds.clone();
            let new_amount = Amount::fake();
            updated_bonds.iter_mut().for_each(|bond| {
                bond.amount = new_amount.clone();
            });

            insert_bonds(conn, updated_bonds)?;

            let queried_bonds = query_bonds(conn);

            assert_eq!(queried_bonds.len(), fake_bonds_len);
            assert_eq!(
                queried_bonds
                    .into_iter()
                    .map(|b| Amount::from(b.raw_amount))
                    .collect::<Vec<_>>(),
                vec![new_amount; fake_bonds_len]
            );

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_unbonds function correctly handles empty unbonds
    /// input.
    #[tokio::test]
    async fn test_insert_unbonds_with_empty_unbonds() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_unbonds_len = 10;
            let fake_validator = Validator::fake();
            let fake_unbonds: Vec<Unbond> = (0..fake_unbonds_len)
                .map(|_| Unbond::fake(fake_validator.clone().address))
                .collect();
            seed_unbonds(conn, fake_validator, fake_unbonds)?;

            insert_unbonds(conn, vec![])?;

            let queried_bonds = query_unbonds(conn);

            assert_eq!(queried_bonds.len(), fake_unbonds_len);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_unbonds function panics if validator is not in db.
    #[tokio::test]
    #[should_panic]
    async fn test_insert_unbonds_with_missing_validator() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_unbonds: Vec<Unbond> = (0..10)
                .map(|_| Unbond::fake(fake_validator.clone().address))
                .collect();

            insert_unbonds(conn, fake_unbonds)?;

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_unbonds function correctly inserts unbonds into the
    /// empty db.
    #[tokio::test]
    async fn test_insert_unbonds_with_empty_db() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_unbonds_len = 10;
            let fake_unbonds: Vec<Unbond> = (0..fake_unbonds_len)
                .map(|_| Unbond::fake(fake_validator.clone().address))
                .collect();

            seed_validator(conn, fake_validator)?;

            insert_unbonds(conn, fake_unbonds)?;

            let queried_unbonds = query_unbonds(conn);

            assert_eq!(queried_unbonds.len(), fake_unbonds_len);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    /// Test that the insert_unbonds function updates the raw_amount on conflict
    #[tokio::test]
    async fn test_insert_unbonds_with_conflict() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let fake_validator = Validator::fake();
            let fake_unbonds_len = 10;
            let fake_unbonds: Vec<Unbond> = (0..fake_unbonds_len)
                .map(|_| Unbond::fake(fake_validator.clone().address))
                .collect();

            seed_unbonds(conn, fake_validator.clone(), fake_unbonds.clone())?;

            let mut updated_unbonds = fake_unbonds.clone();
            let new_amount = Amount::fake();
            updated_unbonds.iter_mut().for_each(|unbond| {
                unbond.amount = new_amount.clone();
            });

            insert_unbonds(conn, updated_unbonds)?;

            let queried_unbonds = query_unbonds(conn);
            let queried_unbonds_len = queried_unbonds.len();

            assert_eq!(queried_unbonds_len, fake_unbonds_len);
            assert_eq!(
                queried_unbonds
                    .into_iter()
                    .map(|b| Amount::from(b.raw_amount))
                    .collect::<Vec<_>>(),
                vec![new_amount; queried_unbonds_len]
            );

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    fn seed_bonds(
        conn: &mut PgConnection,
        validator: Validator,
        balances: Bonds,
    ) -> anyhow::Result<()> {
        let validator: ValidatorDb = diesel::insert_into(validators::table)
            .values(ValidatorInsertDb::from_validator(validator))
            .get_result(conn)
            .context("Failed to insert validator")?;

        diesel::insert_into(bonds::table)
            .values::<&Vec<BondInsertDb>>(
                &balances
                    .into_iter()
                    .map(|bond| BondInsertDb::from_bond(bond, validator.id))
                    .collect::<Vec<_>>(),
            )
            .execute(conn)
            .context("Failed to update balances in db")?;

        anyhow::Ok(())
    }

    fn seed_unbonds(
        conn: &mut PgConnection,
        validator: Validator,
        balances: Unbonds,
    ) -> anyhow::Result<()> {
        let validator: ValidatorDb = diesel::insert_into(validators::table)
            .values(ValidatorInsertDb::from_validator(validator))
            .get_result(conn)
            .context("Failed to insert validator")?;

        diesel::insert_into(unbonds::table)
            .values::<&Vec<UnbondInsertDb>>(
                &balances
                    .into_iter()
                    .map(|unbond| {
                        UnbondInsertDb::from_unbond(unbond, validator.id)
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn)
            .context("Failed to update balances in db")?;

        anyhow::Ok(())
    }

    fn seed_validator(
        conn: &mut PgConnection,
        validator: Validator,
    ) -> anyhow::Result<()> {
        diesel::insert_into(validators::table)
            .values(ValidatorInsertDb::from_validator(validator))
            .execute(conn)
            .context("Failed to insert validator")?;

        anyhow::Ok(())
    }

    fn query_bonds(conn: &mut PgConnection) -> Vec<BondDb> {
        bonds::table
            .select(BondDb::as_select())
            .load::<BondDb>(conn)
            .expect("Failed to query bonds")
    }

    fn query_unbonds(conn: &mut PgConnection) -> Vec<UnbondDb> {
        unbonds::table
            .select(UnbondDb::as_select())
            .load::<UnbondDb>(conn)
            .expect("Failed to query bonds")
    }
}
