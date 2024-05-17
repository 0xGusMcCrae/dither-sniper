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
    quote_response: QuoteResponse,
    user_public_key: String,
    wrap_and_unwrap_sol: bool,
    // #[serde(skip_serializing_if = "Option::is_none")]
    fee_account: Option<String>,
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

pub struct JupAg {
    client: Client,
    rpc: RpcClient,
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
            rpc,
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
        // println!("Serialized JSON body: {}", json_body);

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
            
        println!("Priority fee: {:?}", swap_response.prioritization_fee_lamports);
        let new_fee = LAMPORTS_PER_SOL/100; //0.01 sol
        println!("Setting priority fee to: {}", new_fee);
        swap_response.prioritization_fee_lamports = Some(new_fee);

        //decode from base64
        let swap_transaction_buffer = base64::decode(swap_response.swap_transaction)?;
        // println!("Decoded swap transaction buffer length: {}", swap_transaction_buffer.len());
        // println!("Decoded swap transaction buffer: {:?}", swap_transaction_buffer);

        //deserialize transaction
        let mut transaction: VersionedTransaction = bincode::deserialize(&swap_transaction_buffer)?;
        // println!("Deserialized transaction: {:?}", transaction);

        // Print transaction details for debugging
        // println!("\nTransaction message: {:?}", transaction.message);
        // println!("Account keys: {:?}", transaction.message.account_keys);
        // for instruction in &transaction.message.instructions() {
        //     println!("Instruction: {:?}", instruction);
        // }

        // let recent_blockhash = self.rpc.get_latest_blockhash().expect("Couldn't get latest blockhash");
        // transaction.message.set_recent_blockhash(recent_blockhash);
        // let signed_transaction = VersionedTransaction::try_new(transaction.message, &[&self.keypair]).expect("Failed to create new VersionedTransaction");

        // let simulated_response = self.rpc.simulate_transaction(&signed_transaction);
        // println!("\nSimulated tx response: {:?}", simulated_response);
        // let signature = self.rpc.send_and_confirm_transaction_with_spinner(&signed_transaction);
        // println!("\nTransaction signature: {:?}", signature);

        let mut attempts = 0;

        while attempts < 5 {
            let recent_blockhash = self.rpc.get_latest_blockhash().expect("Couldn't get latest blockhash");
            println!("blockhash: {recent_blockhash}");

            let mut message_clone = transaction.message.clone();
            message_clone.set_recent_blockhash(recent_blockhash);

            let signed_transaction = VersionedTransaction::try_new(message_clone, &[&self.keypair])
                .expect("Failed to create new VersionedTransaction");
            
            println!("ok we finna make a tx");

            match self.rpc.send_and_confirm_transaction(&signed_transaction) {
                Ok(signature) => {
                    println!("Transaction signature: {}", signature);
                    return Ok(());
                },
                Err(err) => {
                    println!("Transaction failed with error: {}. Retrying... ({}/{})", err, attempts+1, 5);
                    attempts += 1;
                    sleep(Duration::from_secs(1)).await; // Wait before retrying
                },
            }
        }

        Err(MyError::SolanaClient(ClientError {
            kind: ClientErrorKind::Custom("Max retries reached".to_string()),
            request: None,
        }))

            
    }
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    
    let jup = JupAg::new()?;

    let in_token = "So11111111111111111111111111111111111111112";
    let out_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let in_amount = 1000000;
    let slippage = 3000;

    let quote_response = jup.get_quote(in_token, out_token, in_amount, slippage).await?;
        
    let my_pubkey: Pubkey = jup.keypair.try_pubkey().unwrap();
    let pubkey_string = my_pubkey.to_string();
    // println!("pubkey: {:?}, pubkey string: {:?}", my_pubkey, pubkey_string);
    let swap_transaction = SwapTransaction {
        quote_response,
        user_public_key: pubkey_string,
        wrap_and_unwrap_sol: true,
        fee_account: None,
    };

    //DEBUGGING
    let balance = jup.rpc.get_balance(&jup.keypair.pubkey())?;
    println!("\nFee payer balance: {}", balance);

    jup.execute_swap(&swap_transaction).await?;

    Ok(())   
}

