use std::str::FromStr;
use std::time::Duration;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, B256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolEvent,
};
use dotenv::dotenv;
use futures_util::StreamExt;
use tokio::time::sleep;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "abi/ERC20.json"
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let mut attempts = 0;
    const MAX_RETRIES: usize = 10;

    while attempts < MAX_RETRIES {
        attempts += 1;
        match run().await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("Error occurred: {e}. Attempt {attempts}/{MAX_RETRIES}. Retrying...");
                if attempts < MAX_RETRIES {
                    sleep(Duration::from_secs(attempts as u64 * 2)).await;
                } else {
                    eprintln!("Max retries reached. Exiting...");
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}

async fn run() -> eyre::Result<()> {
    let topic = std::env::var("TOPIC")?;
    let rpc_url = std::env::var("WEBSOCKET_RPC")?;
    let private_key = std::env::var("PRIVATE_KEY")?;
    let admin_address = std::env::var("ADMIN_ADDRESS")?;

    let topic2 = B256::from_str(&topic)?;
    let admin_wallet = Address::from_str(&admin_address)?;

    let signer = PrivateKeySigner::from_str(&private_key)?;
    let wallet = EthereumWallet::from(signer);

    let ws = WsConnect::new(rpc_url);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_ws(ws)
        .await?;

    let filter = Filter::new()
        .topic2(topic2)
        .event_signature(ERC20::Transfer::SIGNATURE_HASH);

    let stream = &mut provider.subscribe_logs(&filter).await?.into_stream();

    while let Some(log) = stream.next().await {
        match log.topic0() {
            Some(&ERC20::Transfer::SIGNATURE_HASH) => {
                let ERC20::Transfer {
                    value, from, to, ..
                } = log.log_decode()?.inner.data;
                let contract = ERC20::new(log.address(), &provider);
                let balance = contract.balanceOf(to).call().await?;
                let value = std::cmp::max(value, balance._0);
                println!("Sending {value} from {from} to {admin_wallet}");

                let tx_receipt = contract
                    .transfer(admin_wallet, value)
                    .send()
                    .await?
                    .get_receipt()
                    .await?;

                let hash = tx_receipt.transaction_hash;

                println!("Sent transaction: {hash}");
            }
            _ => (),
        }
    }
    Ok(())
}
