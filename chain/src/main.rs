use std::collections::HashSet;
use std::convert::identity;
use std::sync::Arc;

use anyhow::Context;
use chain::app_state::AppState;
use chain::config::AppConfig;
use chain::repository;
use chain::services::namada::{
    query_all_balances, query_all_bonds_and_unbonds, query_all_proposals,
    query_bonds, query_last_block_height, query_redelegations, query_tokens,
};
use chain::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service,
};
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use deadpool_diesel::postgres::Object;
use futures::future::FutureExt;
use futures::stream::StreamExt;
use namada_sdk::time::DateTimeUtc;
use orm::migrations::CustomMigrationSource;
use repository::pgf as namada_pgf_repository;
use shared::balance::TokenSupply;
use shared::block::Block;
use shared::block_result::BlockResult;
use shared::checksums::Checksums;
use shared::cometbft::CometbftBlock;
use shared::crawler::crawl;
use shared::crawler_state::ChainCrawlerState;
use shared::error::{
    AsDbError, AsRpcError, AsTaskJoinError, ContextDbInteractError, MainError,
};
use shared::futures::AwaitContainer;
use shared::id::Id;
use shared::token::Token;
use shared::utils::BalanceChange;
use shared::validator::ValidatorSet;
use tendermint_rpc::HttpClient;
use tendermint_rpc::client::CompatMode;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;
use tokio::time::Instant;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    let client =
        HttpClient::builder(config.tendermint_url.as_str().parse().unwrap())
            .compat_mode(CompatMode::V0_37)
            .build()
            .unwrap();

    let chain_id = tendermint_service::query_status(&client)
        .await
        .into_rpc_error()?
        .node_info
        .network
        .to_string();

    tracing::info!("Network chain id: {}", chain_id);

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = namada_service::query_tx_code_hash(&client, &code_path)
            .await
            .unwrap_or_else(|| {
                panic!("{} must be defined in namada storage.", code_path)
            });
        checksums.add(code_path, code.to_lowercase());
    }

    config.log.init();

    let client = Arc::new(client);

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    CustomMigrationSource::new(chain_id)
        .run_migrations(&conn)
        .await
        .expect("Should be able to run migrations");

    rlimit::increase_nofile_limit(10240).unwrap();
    rlimit::increase_nofile_limit(u64::MAX).unwrap();

    let last_block_height = namada_service::get_last_block(&client)
        .await
        .into_rpc_error()?;
    let crawler_state = db_service::try_get_chain_crawler_state(&conn)
        .await
        .into_db_error()?;

    let limit = last_block_height - config.storage_read_past_height_limit;

    let can_continue =
        crawler_state.map(|s| (s.last_processed_block >= limit, s));
    let can_backfill = config.backfill_from.map(|bf| (bf >= limit, bf));

    // See if we can start from existing crawler_state
    let crawler_state = match (can_backfill, can_continue) {
        (Some((true, height)), _) => {
            tracing::warn!("Backfilling from block height {}", height);
            Some(ChainCrawlerState {
                last_processed_block: height,
                last_processed_epoch: 0,
                first_block_in_epoch: 0,
                timestamp: 0,
            })
        }
        (None, Some((true, crawler_state))) => {
            tracing::info!(
                "Found chain crawler state, attempting initial crawl at block \
                 {}...",
                crawler_state.last_processed_block
            );

            // Try to run crawler_fn with the last processed block
            let crawl_result = crawling_fn(
                crawler_state.last_processed_block,
                client.clone(),
                conn.clone(),
                checksums.clone(),
                true,
            )
            .await;

            match crawl_result {
                Err(MainError::RpcError) => {
                    // If there was an RpcError, it likely means the block was
                    // pruned from the node. We need to do
                    // an initial_query in that case.
                    tracing::error!(
                        "Failed to query block {}, starting from \
                         initial_query ...",
                        crawler_state.last_processed_block,
                    );
                    None
                }
                Err(_) => {
                    // If any other type of error occurred, we should not
                    // increment last_processed_block but
                    // crawl from there without initial_query
                    tracing::info!(
                        "Initial crawl had an error (not RpcError), \
                         continuing from block {}...",
                        crawler_state.last_processed_block
                    );
                    Some(crawler_state)
                }
                Ok(_) => {
                    // If the crawl was successful, increment last_processed
                    // block and continue from there.
                    let next_block = crawler_state.last_processed_block + 1;
                    tracing::info!(
                        "Initial crawl was successful, continuing from block \
                         {}...",
                        next_block
                    );
                    Some(ChainCrawlerState {
                        last_processed_block: next_block,
                        ..crawler_state
                    })
                }
            }
        }
        _ => {
            tracing::info!(
                "Couldn't continue from the last state. Either state does not \
                 exist or it's more than {} blocks behind. Starting from \
                 initial_query...",
                config.storage_read_past_height_limit
            );
            None
        }
    };

    // Handle cases where we need to perform initial query
    let crawler_state = match crawler_state {
        Some(state) => state,
        None => {
            initial_query(
                &client,
                &conn,
                checksums.clone(),
                config.initial_query_retry_time,
                config.initial_query_retry_attempts,
            )
            .await?;

            db_service::get_chain_crawler_state(&conn)
                .await
                .into_db_error()?
        }
    };

    crawl(
        move |block_height| {
            crawling_fn(
                block_height,
                client.clone(),
                conn.clone(),
                checksums.clone(),
                config.backfill_from.is_none(),
            )
        },
        crawler_state.last_processed_block,
        Some(1000),
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
    checksums: Checksums,
    should_update_crawler_state: bool,
) -> Result<(), MainError> {
    let should_process = can_process(block_height, client.clone()).await?;

    if !should_process {
        let timestamp = Utc::now().naive_utc();
        update_crawler_timestamp(&conn, timestamp).await?;

        tracing::trace!(
            block = block_height,
            "Block does not exist yet, waiting...",
        );

        return Err(MainError::NoAction);
    }

    let start = Instant::now();

    let cometbft_block =
        get_cometbft_block_with_fallback(&conn, &client, block_height)
            .await
            .into_db_error()?;

    tracing::debug!(block = block_height, "Query first block in epoch...");
    let first_block_in_epoch =
        namada_service::get_first_block_in_epoch(&client)
            .await
            .into_rpc_error()?;

    let native_token = namada_service::get_native_token(&client)
        .await
        .into_rpc_error()?;
    let native_token_address: namada_sdk::address::Address =
        native_token.clone().into();

    let (block, tm_block_response, epoch) =
        get_block(cometbft_block, &client, checksums, &native_token_address)
            .await?;

    let rate_limits = first_block_in_epoch.eq(&block_height).then(|| {
        let client = Arc::clone(&client);

        // start this series of queries in parallel, which take
        // quite a while
        tokio::spawn(async move {
            let tokens = query_tokens(&client)
                .await?
                .into_iter()
                .map(|token| token.to_string());

            namada_service::get_rate_limits_for_tokens(&client, tokens, epoch)
                .await
        })
    });

    tracing::info!(
        block = block_height,
        txs = block.transactions.len(),
        "Deserialized {} txs...",
        block.transactions.len()
    );

    let ibc_tokens = block
        .ibc_tokens()
        .into_iter()
        .map(Token::Ibc)
        .collect::<Vec<Token>>();

    let native_addresses =
        namada_service::query_native_addresses_balance_change(Token::Native(
            native_token.clone(),
        ));
    let addresses = block.addresses_with_balance_change(&native_token);

    let token_supplies = first_block_in_epoch
        .eq(&block_height)
        .then(|| query_token_supplies(&client, &conn, &native_token, epoch))
        .future()
        .await
        .transpose()?;

    let validators_addresses = if first_block_in_epoch.eq(&block_height) {
        let previous_epoch = epoch.saturating_sub(1);
        namada_service::get_all_consensus_validators_addresses_at(
            &client,
            previous_epoch,
            native_token.clone(),
        )
        .await
        .into_rpc_error()?
    } else {
        HashSet::default()
    };

    let block_proposer_address = block
        .header
        .proposer_address_namada
        .as_ref()
        .map(|address| BalanceChange {
            address: Id::Account(address.clone()),
            token: Token::Native(native_token.clone()),
        });

    let pgf_receipient_addresses = if first_block_in_epoch.eq(&block_height) {
        conn.interact(move |conn| {
            namada_pgf_repository::get_pgf_receipients_balance_changes(
                conn,
                &native_token,
            )
        })
        .await
        .context_db_interact_error()
        .and_then(identity)
        .into_db_error()?
    } else {
        HashSet::default()
    };

    let all_balance_changed_addresses = addresses
        .iter()
        .chain(block_proposer_address.iter())
        .chain(pgf_receipient_addresses.iter())
        .chain(validators_addresses.iter())
        .chain(native_addresses.iter())
        .cloned()
        .collect::<HashSet<_>>();

    let balances = namada_service::query_balance(
        &client,
        &all_balance_changed_addresses,
        block_height,
    )
    .await
    .into_rpc_error()?;

    tracing::debug!(
        block = block_height,
        addresses = all_balance_changed_addresses.len(),
        "Updating balance for {} addresses...",
        all_balance_changed_addresses.len()
    );

    let next_governance_proposal_id =
        namada_service::query_next_governance_id(&client, block_height)
            .await
            .into_rpc_error()?;

    let proposals = block.governance_proposal(next_governance_proposal_id);
    tracing::debug!(
        block = block_height,
        "Creating {} governance proposals...",
        proposals.len()
    );

    let proposals_with_tally =
        namada_service::query_tallies(&client, proposals)
            .await
            .into_rpc_error()?;

    let proposals_votes = block.governance_votes();
    tracing::debug!(
        block = block_height,
        "Creating {} governance votes...",
        proposals_votes.len()
    );

    let validators = block.new_validators();
    let validator_set = ValidatorSet {
        validators: validators.clone(),
        epoch,
    };

    let validators_state_change = block.update_validators_state();
    tracing::debug!(
        "Updating {} validators state",
        validators_state_change.len()
    );

    let addresses = block.bond_addresses();
    let bonds = query_bonds(&client, &addresses).await.into_rpc_error()?;
    tracing::debug!(
        block = block_height,
        "Updating bonds for {} addresses",
        bonds.len()
    );
    let redelegations = query_redelegations(&client, &addresses)
        .await
        .into_rpc_error()?;
    tracing::debug!("Updating redelegations for {} addresses", bonds.len());

    let bonds_updates = bonds
        .iter()
        .cloned()
        .filter_map(|(_, _, bond)| bond)
        .collect::<Vec<_>>();

    let removed_bonds_addresses = bonds
        .iter()
        .cloned()
        .map(|(source, validator, _)| (source, validator))
        .collect::<Vec<(Id, Id)>>();

    let addresses = block.unbond_addresses();
    let unbonds = namada_service::query_unbonds(&client, addresses)
        .await
        .into_rpc_error()?;
    tracing::debug!(
        block = block_height,
        "Updating unbonds for {} addresses",
        unbonds.len()
    );

    let withdraw_addreses = block.withdraw_addresses();

    let revealed_pks = block.revealed_pks();
    tracing::debug!(
        block = block_height,
        "Updating revealed pks for {} addresses",
        revealed_pks.len()
    );

    let metadata_change = block.validator_metadata();

    let reward_claimers = block.pos_rewards();

    let crawler_state = ChainCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        first_block_in_epoch,
        timestamp: chrono::Utc::now().timestamp(),
    };

    let rate_limits =
        rate_limits
            .future()
            .await
            .map_or(Ok(vec![]), |maybe_rate_limits| {
                maybe_rate_limits
                    .context("Failed to await on rate limits query")
                    .into_task_join_error()?
                    .into_rpc_error()
            })?;

    let first_checkpoint = Instant::now();

    tracing::info!(
        txs = block.transactions.len(),
        ibc_tokens = ibc_tokens.len(),
        balance_changes = balances.len(),
        proposals = proposals_with_tally.len(),
        votes = proposals_votes.len(),
        validators = validators.len(),
        bonds = bonds_updates.len(),
        unbonds = unbonds.len(),
        withdraws = withdraw_addreses.len(),
        claimed_rewards = reward_claimers.len(),
        revealed_pks = revealed_pks.len(),
        validator_state = validators_state_change.len(),
        epoch = epoch,
        first_block_in_epoch = first_block_in_epoch,
        block = block_height,
        time_taken = first_checkpoint.duration_since(start).as_secs_f64(),
        "Queried block successfully",
    );

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_tokens(
                    transaction_conn,
                    ibc_tokens,
                )?;

                repository::balance::insert_token_supplies(
                    transaction_conn,
                    token_supplies.into_iter().flatten(),
                )?;

                repository::balance::insert_ibc_rate_limits(
                    transaction_conn,
                    rate_limits,
                )?;

                repository::block::upsert_block(
                    transaction_conn,
                    block,
                    tm_block_response,
                )?;

                repository::balance::insert_balances(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(
                    transaction_conn,
                    proposals_with_tally,
                )?;
                repository::gov::insert_votes(
                    transaction_conn,
                    proposals_votes,
                )?;

                repository::pos::upsert_validators(
                    transaction_conn,
                    validator_set,
                )?;

                repository::pos::upsert_validator_state(
                    transaction_conn,
                    validators_state_change,
                )?;

                // We first remove all the bonds and then insert the new ones
                repository::pos::clear_bonds(
                    transaction_conn,
                    removed_bonds_addresses,
                )?;
                repository::pos::insert_bonds(transaction_conn, bonds_updates)?;

                repository::pos::insert_unbonds(transaction_conn, unbonds)?;
                repository::pos::insert_redelegations(
                    transaction_conn,
                    redelegations,
                )?;
                repository::pos::remove_withdraws(
                    transaction_conn,
                    epoch,
                    withdraw_addreses,
                )?;

                repository::pos::delete_claimed_rewards(
                    transaction_conn,
                    reward_claimers,
                )?;

                repository::pos::update_validator_metadata(
                    transaction_conn,
                    metadata_change,
                )?;

                repository::revealed_pk::insert_revealed_pks(
                    transaction_conn,
                    revealed_pks,
                )?;

                if should_update_crawler_state {
                    repository::crawler_state::upsert_crawler_state(
                        transaction_conn,
                        crawler_state,
                    )?;
                }

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()?;

    let second_checkpoint = Instant::now();

    tracing::info!(
        block = block_height,
        time_taken = second_checkpoint
            .duration_since(first_checkpoint)
            .as_secs_f64(),
        "Inserted block into database"
    );

    Ok(())
}

async fn initial_query(
    client: &HttpClient,
    conn: &Object,
    checksums: Checksums,
    retry_time: u64,
    retry_attempts: usize,
) -> Result<(), MainError> {
    let retry_strategy = ExponentialBackoff::from_millis(retry_time)
        .map(jitter)
        .take(retry_attempts);
    Retry::spawn(retry_strategy, || {
        try_initial_query(client, conn, checksums.clone())
    })
    .await
}

async fn try_initial_query(
    client: &HttpClient,
    conn: &Object,
    checksums: Checksums,
) -> Result<(), MainError> {
    tracing::debug!("Querying initial data...");
    let block_height =
        query_last_block_height(client).await.into_rpc_error()?;
    let first_block_in_epoch = namada_service::get_first_block_in_epoch(client)
        .await
        .into_rpc_error()?;

    let native_token: namada_sdk::address::Address =
        namada_service::get_native_token(client)
            .await
            .into_rpc_error()?
            .into();

    let cometbft_block =
        get_cometbft_block_with_fallback(conn, client, block_height)
            .await
            .into_db_error()?;

    let (block, tm_block_response, epoch) =
        get_block(cometbft_block, client, checksums.clone(), &native_token)
            .await?;

    let tokens = query_tokens(client).await.into_rpc_error()?;

    let rate_limits_fut = async {
        namada_service::get_rate_limits_for_tokens(
            client,
            tokens.iter().map(|token| token.to_string()),
            epoch,
        )
        .await
        .into_rpc_error()
    };
    let token_supplies_fut = async {
        let native_token = namada_service::get_native_token(client)
            .await
            .into_rpc_error()?;
        query_token_supplies(client, conn, &native_token, epoch).await
    };

    let (rate_limits, token_supplies) =
        futures::try_join!(rate_limits_fut, token_supplies_fut)?;

    // This can sometimes fail if the last block height in the node has moved
    // forward after we queried for it. In that case, query_all_balances
    // returns an Err indicating that it can only be used for
    // the last block. This function will be retried in that case.
    let balances = query_all_balances(client, block_height)
        .await
        .into_rpc_error()?;

    tracing::debug!(block = block_height, "Querying validators set...");
    let pipeline_length = namada_service::query_pipeline_length(client)
        .await
        .into_rpc_error()?;
    // We need to add pipeline_length to the epoch as it is possible to bond in
    // advance
    let validator_set = namada_service::get_validator_set_at_epoch(
        client,
        epoch + pipeline_length as u32,
    )
    .await
    .into_rpc_error()?;

    let validators_set =
        namada_service::get_validator_addresses_at_epoch(client, epoch)
            .await
            .into_rpc_error()?;

    tracing::debug!("Querying redelegations...");
    let redelegations =
        namada_service::query_all_redelegations(client, validators_set)
            .await
            .into_rpc_error()?;

    tracing::debug!("Querying bonds and unbonds...");
    let (bonds, unbonds) = query_all_bonds_and_unbonds(client, None, None)
        .await
        .into_rpc_error()?;

    tracing::debug!(block = block_height, "Querying proposals...");
    let proposals = query_all_proposals(client).await.into_rpc_error()?;
    let proposals_with_tally =
        namada_service::query_tallies(client, proposals.clone())
            .await
            .into_rpc_error()?;

    let proposals_votes = namada_service::query_all_votes(
        client,
        proposals.iter().map(|p| p.id).collect(),
    )
    .await
    .into_rpc_error()?;

    let timestamp = DateTimeUtc::now().0.timestamp();

    let crawler_state = ChainCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        first_block_in_epoch,
        timestamp,
    };

    tracing::debug!(block = block_height, "Inserting initial data...");

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_tokens(transaction_conn, tokens)?;

                repository::block::upsert_block(
                    transaction_conn,
                    block,
                    tm_block_response,
                )?;

                repository::balance::insert_token_supplies(
                    transaction_conn,
                    token_supplies,
                )?;

                repository::balance::insert_ibc_rate_limits(
                    transaction_conn,
                    rate_limits,
                )?;

                tracing::debug!(
                    block = block_height,
                    "Inserting {} balances...",
                    balances.len()
                );
                repository::balance::insert_balances(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(
                    transaction_conn,
                    proposals_with_tally,
                )?;

                repository::gov::insert_votes(
                    transaction_conn,
                    proposals_votes,
                )?;

                repository::pos::upsert_validators(
                    transaction_conn,
                    validator_set,
                )?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;
                repository::pos::insert_redelegations(
                    transaction_conn,
                    redelegations,
                )?;

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
                    crawler_state,
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()
}

async fn can_process(
    block_height: u32,
    client: Arc<HttpClient>,
) -> Result<bool, MainError> {
    let last_block_height = namada_service::query_last_block_height(&client)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to query Namada's last committed block: {}",
                e
            );
            MainError::RpcError
        })?;

    Ok(last_block_height >= block_height)
}

async fn update_crawler_timestamp(
    conn: &Object,
    timestamp: NaiveDateTime,
) -> Result<(), MainError> {
    conn.interact(move |transaction_conn| {
        repository::crawler_state::update_crawler_timestamp(
            transaction_conn,
            timestamp,
        )?;

        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()
}

async fn get_block(
    block: CometbftBlock,
    client: &HttpClient,
    checksums: Checksums,
    native_token: &namada_sdk::address::Address,
) -> Result<(Block, TendermintBlockResponse, u32), MainError> {
    let block_height = block.block_height;

    tracing::debug!(block = block_height, "Query block...");
    let tm_block_response = block.block;
    tracing::debug!(
        block = block_height,
        "Raw block contains {} txs...",
        tm_block_response.block.data.len()
    );

    tracing::debug!(block = block_height, "Query block results...");
    let tm_block_results_response = block.events;
    let block_results = BlockResult::from(tm_block_results_response);

    tracing::debug!(block = block_height, "Query epoch...");
    let epoch = namada_service::get_epoch_at_block_height(client, block_height)
        .await
        .into_rpc_error()?;

    let proposer_address_namada = namada_service::get_validator_namada_address(
        client,
        &Id::from(&tm_block_response.block.header.proposer_address),
    )
    .await
    .into_rpc_error()?;

    tracing::debug!(
        block = block_height,
        tm_address = tm_block_response.block.header.proposer_address.to_string(),
        namada_address = ?proposer_address_namada,
        "Got block proposer address"
    );

    let block = Block::from(
        &tm_block_response,
        &block_results,
        &proposer_address_namada,
        checksums,
        epoch,
        block_height,
        native_token,
    );

    Ok((block, tm_block_response, epoch))
}

async fn query_non_native_supplies(
    client: &HttpClient,
    conn: &Object,
    epoch: u32,
) -> Result<Vec<TokenSupply>, MainError> {
    let token_addresses = db_service::get_non_native_tokens(conn)
        .await
        .into_db_error()?;

    let mut buffer = Vec::with_capacity(1);

    let mut stream = futures::stream::iter(token_addresses)
        .map(|address| async move {
            namada_service::get_token_supply(client, address, epoch)
                .await
                .into_rpc_error()
        })
        .buffer_unordered(32);

    while let Some(maybe_supply) = stream.next().await {
        let supply = maybe_supply?;
        buffer.push(supply);
    }

    Ok(buffer)
}

async fn query_token_supplies(
    client: &HttpClient,
    conn: &Object,
    native_token: &Id,
    epoch: u32,
) -> Result<Vec<TokenSupply>, MainError> {
    let native_fut =
        namada_service::get_native_token_supply(client, native_token, epoch)
            .map(|result| result.into_rpc_error());

    let non_native_fut = query_non_native_supplies(client, conn, epoch);

    let (native, non_native) = futures::try_join!(native_fut, non_native_fut)?;

    let mut supplies = non_native;
    supplies.push(native);

    Ok(supplies)
}

pub async fn get_cometbft_block_with_fallback(
    conn: &Object,
    client: &HttpClient,
    block_height: u32,
) -> anyhow::Result<CometbftBlock> {
    let block = repository::cometbft::get_block(conn, block_height)
        .await
        .context("Failed to get block")?;

    let block = match block {
        Some(block) => block,
        None => {
            let block = tendermint_service::query_raw_block_at_height(
                client,
                block_height,
            )
            .await
            .context("Failed to query block")?;

            let events = tendermint_service::query_raw_block_results_at_height(
                client,
                block_height,
            )
            .await
            .context("Failed to query block results")?;

            CometbftBlock {
                block_height,
                block,
                events,
            }
        }
    };

    Ok(block)
}
