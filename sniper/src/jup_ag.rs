use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
    client::SyncClient,
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
use std::{
    env,
    sync::Arc,
};
use thiserror::Error;
use tokio;
use tokio::{
    time::{sleep, Duration},
    sync::Mutex,
};
use bincode;
use borsh::BorshDeserialize;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Solana transport error: {0}")]
    SolanaTransport(#[from] TransportError),
    #[error("Solana client error: {0}")]
    SolanaClient(#[from] ClientError),
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    swap_transaction: String,
    last_valid_block_height: u64,
    prioritization_fee_lamports: Option<u64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapTransaction {
    pub quote_response: QuoteResponse,
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: bool,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    input_mint: String,
    in_amount: String,
    output_mint: String,
    out_amount: String,
    other_amount_threshold: String,
    swap_mode: String,
    slippage_bps: u64,
    platform_fee: Option<String>,
    price_impact_pct: String,
    route_plan: Vec<RoutePlan>,
    context_slot: Option<u64>,
    time_taken: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    swap_info: SwapInfo,
    percent: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    amm_key: String,
    label: Option<String>,
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    fee_amount: String,
    fee_mint: String,
}

// #[derive(Clone)]
pub struct JupAg {
    client: Client,
    rpc: Arc<Mutex<RpcClient>>,
    pub keypair: Keypair,
}

impl JupAg {
    pub fn new() -> Result<Self, MyError> {
        dotenv::dotenv().ok();

        let client = Client::new();

        let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
        let rpc: RpcClient = RpcClient::new_with_commitment(&rpc_url, CommitmentConfig::confirmed());

        let keypair_path = env::var("KEYPAIR_PATH").expect("KEYPAIR_PATH must be set");
        let keypair: Keypair = read_keypair_file(keypair_path).expect("Failed to read keypair file");

        Ok(JupAg {
            client,
            rpc: Arc::new(Mutex::new(rpc)),
            keypair,
        })
    }

    pub async fn get_quote(&self, input_token_address: &str, output_token_address: &str, input_amount: u64, slippage_bps: u32) -> Result<QuoteResponse, MyError> {
        
        let quote_url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            input_token_address,
            output_token_address,
            input_amount,
            slippage_bps
        );
    
        let response: QuoteResponse = self.client
            .get(&quote_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
    
        Ok(response)
    }

    pub async fn execute_swap(&self, swap_transaction: &SwapTransaction) -> Result<(), MyError> {
        let json_body = serde_json::to_string(&swap_transaction)?;

        let swap_url = format!("https://quote-api.jup.ag/v6/swap");
        
        let mut swap_response: SwapResponse = self.client
            .post(swap_url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
            
        // println!("Priority fee: {:?}", swap_response.prioritization_fee_lamports);
        let new_fee = LAMPORTS_PER_SOL/200; //0.005 sol
        // println!("Setting priority fee to: {}", new_fee);
        swap_response.prioritization_fee_lamports = Some(new_fee);

        //decode from base64
        let swap_transaction_buffer = base64::decode(swap_response.swap_transaction)?;
   
        //deserialize transaction
        let mut transaction: VersionedTransaction = bincode::deserialize(&swap_transaction_buffer)?;
   

        let mut attempts = 0;

        while attempts < 3 {
            let recent_blockhash = self.rpc.lock().await.get_latest_blockhash().expect("Couldn't get latest blockhash");
            // println!("blockhash: {recent_blockhash}");

            let mut message_clone = transaction.message.clone();
            message_clone.set_recent_blockhash(recent_blockhash);

            let signed_transaction = VersionedTransaction::try_new(message_clone, &[&self.keypair])
                .expect("Failed to create new VersionedTransaction");
            
            println!("Attempting transaction...");

            // Spam multiple identical transactions with the same blockhash
            let mut handles = Vec::new();
            for _ in 0..10 {
                let rpc_clone = Arc::clone(&self.rpc);
                let tx_clone = signed_transaction.clone();
                handles.push(tokio::spawn(async move {
                    let config = RpcSendTransactionConfig {
                        skip_preflight: true,
                        ..RpcSendTransactionConfig::default()
                    };
                    match rpc_clone.lock().await.send_transaction_with_config(&tx_clone, config) {
                        Ok(signature) => {
                            println!("Transaction signature: {}", signature);
                            Ok(signature)
                        },
                        Err(err) => {
                            println!("Transaction failed with error: {}.", err);
                            Err(err)
                        },
                    }
                }));
                // Short delay between sending transactions
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            for handle in handles {
                match handle.await {
                    Ok(Ok(signature)) => {
                        println!("Attempting to confirm transaction with signature {}", signature);
                        match self.rpc.lock().await.confirm_transaction_with_spinner(
                            &signature,
                            &recent_blockhash,
                            CommitmentConfig::finalized(),
                        ) {
                            Ok(_) => {
                                println!("Transaction confirmed: {}", signature);
                                ()
                            },
                            Err(err) => {
                                println!("Transaction confirmation failed with error: {}", err);
                            }
                        }
                    },
                    Ok(Err(_)) => continue,
                    Err(e) => println!("Join error: {:?}", e),
                }
            }

            // Try another set of txs if none confirmed
            attempts += 1;
            sleep(Duration::from_secs(3)).await; // Wait before retrying
            
        }

        Err(MyError::SolanaClient(ClientError {
            kind: ClientErrorKind::Custom("Max retries reached".to_string()),
            request: None,
        }))

            
    }
}

// #[tokio::main]
// async fn main() -> Result<(), MyError> {
    
//     let jup = JupAg::new()?;

//     let in_token = "So11111111111111111111111111111111111111112";
//     let out_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
//     let in_amount = 1000000;
//     let slippage = 3000;

//     let quote_response = jup.get_quote(in_token, out_token, in_amount, slippage).await?;
        
//     let my_pubkey: Pubkey = jup.keypair.try_pubkey().unwrap();
//     let pubkey_string = my_pubkey.to_string();
//     // println!("pubkey: {:?}, pubkey string: {:?}", my_pubkey, pubkey_string);
//     let swap_transaction = SwapTransaction {
//         quote_response,
//         user_public_key: pubkey_string,
//         wrap_and_unwrap_sol: true,
//         fee_account: None,
//     };

//     //DEBUGGING
//     let balance = jup.rpc.get_balance(&jup.keypair.pubkey())?;
//     println!("\nFee payer balance: {}", balance);

//     jup.execute_swap(&swap_transaction).await?;

//     Ok(())   
// }

