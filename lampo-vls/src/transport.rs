use anyhow::Result;
use lightning_signer::{
    bitcoin::{Address, Network},
    node::NodeServices,
    persist::DummyPersister,
    policy::simple_validator::{make_simple_policy, SimpleValidatorFactory},
    signer::ClockStartingTimeFactory,
    util::{clock::StandardClock, crypto_utils::generate_seed},
};

use std::sync::Arc;
use vls_proxy::vls_protocol_client::Error;
use vls_proxy::{
    vls_protocol_client::Transport,
    vls_protocol_signer::{
        handler::{Handler, HandlerBuilder, RootHandler},
        vls_protocol::model::PubKey,
        vls_protocol::msgs,
    },
};

pub struct LampoNullTransport {
    pub handler: RootHandler,
}

impl LampoNullTransport {
    pub fn new(address: Address, network: Network) -> Arc<Self> {
        let persister = Arc::new(DummyPersister);
        let allowlist = vec![address.to_string()];
        let policy = make_simple_policy(network);
        let validator_factory = Arc::new(SimpleValidatorFactory::new_with_policy(policy));
        let starting_time_factory = ClockStartingTimeFactory::new();
        let clock = Arc::new(StandardClock());
        let services = NodeServices {
            validator_factory,
            starting_time_factory,
            persister,
            clock,
        };
        let seed = generate_seed();
        let builder = HandlerBuilder::new(network, 0, services, seed).allowlist(allowlist);
        let (init_handler, _) = builder.build().expect("Failed to build the RootHandler");
        let handler = init_handler.into_root_handler();
        Arc::new(LampoNullTransport { handler })
    }
}

impl Transport for LampoNullTransport {
    fn node_call(&self, message_ser: Vec<u8>) -> Result<Vec<u8>, Error> {
        let message = msgs::from_vec(message_ser)?;
        let (result, _) = self.handler.handle(message).map_err(|_| Error::Transport)?;
        Ok(result.as_vec())
    }
    fn call(&self, db_id: u64, peer_id: PubKey, message_ser: Vec<u8>) -> Result<Vec<u8>, Error> {
        let message = msgs::from_vec(message_ser)?;
        let handler = self.handler.for_new_client(0, peer_id, db_id);
        let (result, _) = handler.handle(message).map_err(|_| Error::Transport)?;
        Ok(result.as_vec())
    }
}
