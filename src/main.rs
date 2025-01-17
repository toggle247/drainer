use std::str::FromStr;

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

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "artifacts/@openzeppelin/contracts/token/ERC20/ERC20.sol/ERC20.json"
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let topic = std::env::var("TOPIC")?;
    let rpc_url = std::env::var("WEBSOCKET_RPC")?;
    let private_key = std::env::var("PRIVATE_KEY")?;
    let admin_address = std::env::var("ADMIN_ADDRESS")?;

    let topic2 = B256::from_str(&topic)?;
    let admin_wallet = Address::from_str(&admin_address)?;

    let signer = PrivateKeySigner::from_str(&private_key)?;
    let wallet = EthereumWallet::from(signer);

    let ws = WsConnect::new(rpc_url);

    let provider = ProviderBuilder::new().wallet(wallet).on_ws(ws).await?;

    let filter = Filter::new()
        .topic2(topic2)
        .event_signature(ERC20::Transfer::SIGNATURE_HASH);

    let stream = &mut provider.subscribe_logs(&filter).await?.into_stream();

    while let Some(log) = stream.next().await {
        match log.topic0() {
            Some(&ERC20::Transfer::SIGNATURE_HASH) => {
                let ERC20::Transfer { value, from, .. } = log.log_decode()?.inner.data;
                println!("Recieved {value} from {from}");
                let contract = ERC20::new(log.address(), &provider);
                let tx_hash = contract
                    .transfer(admin_wallet, value)
                    .send()
                    .await?
                    .watch()
                    .await?;
                println!("Sent transaction: {tx_hash}");
            }
            _ => (),
        }
    }
    Ok(())
}
