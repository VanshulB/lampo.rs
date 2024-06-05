use async_trait::async_trait;
use std::sync::Arc;
use vls_proxy::vls_protocol_client::{ClientResult, SignerPort, Transport};

pub struct LampoSignerPort {
    pub transport: Arc<dyn Transport>,
}

#[async_trait]
impl SignerPort for LampoSignerPort {
    async fn handle_message(&self, message: Vec<u8>) -> ClientResult<Vec<u8>> {
        self.transport.node_call(message)
    }
    fn is_ready(&self) -> bool {
        true
    }
}
