use std::sync::Arc;

use keys_manager::LampoKeysManager;
use lightning_signer::bitcoin::{Address, Network};
use protocol_handler::LampoVLSInProcess;
use signer::LampoVLSSignerPort;
use triggered::Listener;
use url::Url;
use vls_proxy::portfront::SignerPortFront;
use vls_proxy::vls_frontend::{frontend::SourceFactory, Frontend};
use vls_proxy::vls_protocol_client::{DynSigner, KeysManagerClient, SpendableKeysInterface};

mod keys_manager;
mod protocol_handler;
mod signer;
mod util;

pub(crate) fn make_in_process_signer(
    network: Network,
    lampo_data_dir: String,
    sweep_address: Address,
    bitcoin_rpc_url: Url,
    shutdown_signal: Listener,
) -> Box<dyn SpendableKeysInterface<Signer = DynSigner>> {
    let protocol_handler = Arc::new(LampoVLSInProcess::new(sweep_address.clone(), network));
    let signer_port = Arc::new(LampoVLSSignerPort::new(protocol_handler.clone()));
    let source_factory = Arc::new(SourceFactory::new(lampo_data_dir, network));
    // The SignerPortFront provide a client RPC interface to the core MultiSigner and Node objects via a communications link.
    let signer = Arc::new(SignerPortFront::new(signer_port, network));
    let frontend = Frontend::new(signer, source_factory, bitcoin_rpc_url, shutdown_signal);
    frontend.start();
    let client = KeysManagerClient::new(protocol_handler, network.to_string());
    let keys_manager = LampoKeysManager::new(client, sweep_address);
    Box::new(keys_manager)
}
