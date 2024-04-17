use anyhow::anyhow;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::{fs, path::PathBuf};
use subxt::{
    backend::legacy::LegacyRpcMethods,
    backend::rpc::RpcClient,
    blocks::ExtrinsicEvents,
    config::PolkadotExtrinsicParamsBuilder as Params,
    ext::sp_core::{sr25519, Pair},
    tx::{PairSigner, TxPayload, TxStatus},
    Config, OnlineClient, PolkadotConfig as EntropyConfig,
};

#[derive(Parser, Debug, Clone)]
#[clap(version, about = "CLI tool for uploading entropy programs")]
struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
    /// The chain endpoint to use.
    ///
    /// The format should be in the form of `scheme://hostname:port`.
    ///
    /// Default to `ws://localhost:9944`. If a value exists for `ENTROPY_DEVNET`, that takes
    /// priority.
    #[arg(short, long)]
    chain_endpoint: Option<String>,
}
#[derive(Subcommand, Debug, Clone)]
enum CliCommand {
    /// Store a given program on chain
    StoreProgram {
        /// The menmonic for the deployer key
        mnemonic: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match run_command().await {
        Ok(output) => {
            println!("Success: {}", output.green());
            Ok(())
        }
        Err(err) => {
            println!("{}", "Failed!".red());
            Err(err)
        }
    }
}

async fn run_command() -> anyhow::Result<String> {
    let cli = Cli::parse();

    let endpoint_addr = cli.chain_endpoint.unwrap_or_else(|| {
        std::env::var("ENTROPY_DEVNET").unwrap_or("ws://localhost:9944".to_string())
    });
    let api = get_api(&endpoint_addr).await?;
    let rpc = get_rpc(&endpoint_addr).await?;

    match cli.command {
        CliCommand::StoreProgram {
            mnemonic,
        } => {
            let keypair = <sr25519::Pair as Pair>::from_string(&mnemonic, None).unwrap();
            println!("Uploading program using account: {}", keypair.public());

            let program = include_bytes!("../../target/wasm32-unknown-unknown/release/{{project-name}}.wasm").to_vec();
            let config_interface = fs::read("{{project-name}}_serialized_config_type.txt")?;
            let aux_data_interface = fs::read("{{project-name}}_serialized_aux_data_type.txt")?;

            let hash = store_program(
                &api,
                &rpc,
                &keypair,
                program,
                config_interface,
                aux_data_interface,
            )
            .await?;
            Ok(format!("Program stored {hash}"))
        }
    }
}

pub async fn store_program(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    deployer_pair: &sr25519::Pair,
    program: Vec<u8>,
    configuration_interface: Vec<u8>,
    auxiliary_data_interface: Vec<u8>,
) -> anyhow::Result<<EntropyConfig as Config>::Hash> {
    let update_program_tx = entropy::tx().programs().set_program(
        program,
        configuration_interface,
        auxiliary_data_interface,
    );
    let deployer = PairSigner::<EntropyConfig, sr25519::Pair>::new(deployer_pair.clone());

    let in_block = submit_transaction(api, rpc, &deployer, &update_program_tx, None).await?;
    let result_event = in_block.find_first::<entropy::programs::events::ProgramCreated>()?;
    Ok(result_event
        .ok_or(anyhow!("Error getting program created event"))?
        .program_hash)
}

/// Send a transaction to the Entropy chain
///
/// Optionally takes a nonce, otherwise it grabs the latest nonce from the chain
pub async fn submit_transaction<Call: TxPayload>(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    call: &Call,
    nonce_option: Option<u32>,
) -> anyhow::Result<ExtrinsicEvents<EntropyConfig>> {
    let block_hash = rpc
        .chain_get_block_hash(None)
        .await?
        .ok_or_else(|| anyhow!("Error getting block hash"))?;

    let nonce = if let Some(nonce) = nonce_option {
        nonce
    } else {
        let nonce_call = entropy::apis()
            .account_nonce_api()
            .account_nonce(signer.account_id().clone());
        api.runtime_api().at(block_hash).call(nonce_call).await?
    };

    let latest_block = api.blocks().at_latest().await?;
    let tx_params = Params::new()
        .mortal(latest_block.header(), 32u64)
        .nonce(nonce.into())
        .build();
    let mut tx = api
        .tx()
        .create_signed(call, signer, tx_params)
        .await?
        .submit_and_watch()
        .await?;

    while let Some(status) = tx.next().await {
        match status? {
            TxStatus::InBestBlock(tx_in_block) | TxStatus::InFinalizedBlock(tx_in_block) => {
                return Ok(tx_in_block.wait_for_success().await?);
            }
            TxStatus::Error { message }
            | TxStatus::Invalid { message }
            | TxStatus::Dropped { message } => {
                // Handle any errors:
                return Err(anyhow!("Error submitting tx: {message}"));
            }
            // Continue otherwise:
            _ => continue,
        };
    }
    Err(anyhow!("Error getting event"))
}

#[subxt::subxt(runtime_metadata_path = "entropy_metadata.scale")]
pub mod entropy {}

/// Creates an api instance to talk to chain
/// Chain endpoint set on launch
pub async fn get_api(url: &str) -> Result<OnlineClient<EntropyConfig>, subxt::Error> {
    let api = OnlineClient::<EntropyConfig>::from_url(url).await?;
    Ok(api)
}

/// Creates a rpc instance to talk to chain
/// Chain endpoint set on launch
pub async fn get_rpc(url: &str) -> Result<LegacyRpcMethods<EntropyConfig>, subxt::Error> {
    let rpc_client = RpcClient::from_url(url).await?;
    let rpc_methods = LegacyRpcMethods::<EntropyConfig>::new(rpc_client);
    Ok(rpc_methods)
}
