mod jup_ag;

use jup_ag::{JupAg, SwapTransaction};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
    client::SyncClient,
    // client::Client,
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signer::keypair::read_keypair_file,
    transport::TransportError,
    native_token::LAMPORTS_PER_SOL,
};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
    client_error::{ClientError, ClientErrorKind},
};
use std::env;
use thiserror::Error;
use tokio;
use tokio::time::{sleep, Duration};
use bincode;
use warp::Filter;
use serde_json::Value;
use std::convert::Infallible;

async fn handle_incoming_message(body: Value) -> Result<impl warp::Reply, Infallible> {
    if let Some(out_token) = body.get("address").and_then(Value::as_str) {
        println!("Token address: {}", out_token);

        let jup = JupAg::new().expect("Could not initialize jup");

        let in_token = "So11111111111111111111111111111111111111112"; //solana;
        let in_amount = LAMPORTS_PER_SOL / 100; // 1 sol
        let slippage = 1000; // 10%

        let quote_response = jup.get_quote(in_token, out_token, in_amount, slippage).await.expect("could not get quote");
            
        let my_pubkey: Pubkey = jup.keypair.try_pubkey().unwrap();
        let pubkey_string = my_pubkey.to_string();
        let swap_transaction = SwapTransaction {
            quote_response,
            user_public_key: pubkey_string,
            wrap_and_unwrap_sol: true,
            fee_account: None,
        };

        jup.execute_swap(&swap_transaction).await.expect("could not execute swap");

        Ok(warp::reply::with_status("Trade Executed", warp::http::StatusCode::OK))
    } else {
        println!("Token address not found in request");
        Ok(warp::reply::with_status("Bad Request", warp::http::StatusCode::BAD_REQUEST))
    }
}

#[tokio::main]
async fn main() {
    // Define the endpoint to handle buy calls
    let handle_call = warp::post()
        .and(warp::path("trade"))
        .and(warp::body::json())
        .and_then(handle_incoming_message);

    // Start the Warp server
    warp::serve(handle_call).run(([127,0,0,1], 8080)).await;
}
