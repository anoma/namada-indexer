use std::str::FromStr;

use reqwest::{Client, Url};
use tendermint_rpc::HttpClient;

pub fn build_client(url: &str) -> HttpClient {
    let url = Url::from_str(url).unwrap();
    let inner_client =
        Client::builder().http2_prior_knowledge().build().unwrap();

    HttpClient::new_from_parts(
        inner_client,
        url,
        tendermint_rpc::client::CompatMode::V0_37,
    )
}
