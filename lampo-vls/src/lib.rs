mod keys_manager;
mod signer;
mod transport;
mod util;

use crate::keys_manager::LampoKeysManager;
use crate::signer::LampoSignerPort;
use crate::transport::LampoNullTransport;
use lightning_signer::bitcoin::{Address, Network};
use std::fs;
use std::sync::Arc;
use triggered::Listener;
use url::Url;
use vls_proxy::{
    portfront::SignerPortFront,
    vls_frontend::{frontend::SourceFactory, Frontend},
    vls_protocol_client::{DynSigner, KeysManagerClient, SpendableKeysInterface},
    vls_protocol_signer::handler::Handler,
};

#[allow(dead_code)]
pub(crate) async fn make_null_signer(
    network: Network,
    lampo_data_dir: String,
    sweep_address: Address,
    bitcoin_rpc_url: Url,
    shutdown_signer: Listener,
) -> Box<dyn SpendableKeysInterface<Signer = DynSigner>> {
    let node_id_path = format!("{}/node_id", lampo_data_dir);
    let transport = LampoNullTransport::new(sweep_address.clone(), network);
    let signer_port = Arc::new(LampoSignerPort {
        transport: transport.clone(),
    });
    let source_factory = Arc::new(SourceFactory::new(lampo_data_dir, network));
    let frontend = Frontend::new(
        Arc::new(SignerPortFront::new(signer_port, network)),
        source_factory,
        bitcoin_rpc_url,
        shutdown_signer,
    );
    frontend.start();
    let node_id = transport.handler.node().get_id();
    let client = KeysManagerClient::new(transport, network.to_string());
    let keys_manager = LampoKeysManager {
        client,
        sweep_address,
    };
    fs::write(node_id_path, node_id.to_string()).expect("write node_id");
    Box::new(keys_manager)
}
