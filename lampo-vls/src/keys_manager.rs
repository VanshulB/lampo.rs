use crate::util::create_spending_transaction;
use lightning_signer::{
    bitcoin::{
        secp256k1::{schnorr::Signature, All, Secp256k1},
        Address, Script, Transaction, TxOut, Witness,
    },
    lightning::{
        offers::invoice::UnsignedBolt12Invoice,
        offers::invoice_request::UnsignedInvoiceRequest,
        sign::{EntropySource, NodeSigner, SignerProvider, SpendableOutputDescriptor},
    },
};
use log::info;
use vls_proxy::vls_protocol_client::{DynSigner, KeysManagerClient, SpendableKeysInterface};

trait LampoSigner:
    EntropySource
    + NodeSigner
    + SignerProvider<Signer = DynSigner>
    + SpendableKeysInterface<Signer = DynSigner>
{
}

pub struct LampoKeysManager {
    pub client: KeysManagerClient,
    pub sweep_address: Address,
}

impl LampoSigner for LampoKeysManager {}

impl SignerProvider for LampoKeysManager {
    type Signer = DynSigner;

    fn generate_channel_keys_id(
        &self,
        inbound: bool,
        channel_value_satoshis: u64,
        user_channel_id: u128,
    ) -> [u8; 32] {
        self.client
            .generate_channel_keys_id(inbound, channel_value_satoshis, user_channel_id)
    }

    fn derive_channel_signer(
        &self,
        channel_value_satoshis: u64,
        channel_keys_id: [u8; 32],
    ) -> Self::Signer {
        let client = self
            .client
            .derive_channel_signer(channel_value_satoshis, channel_keys_id);
        DynSigner::new(client)
    }

    fn read_chan_signer(
        &self,
        reader: &[u8],
    ) -> Result<Self::Signer, lightning_signer::lightning::ln::msgs::DecodeError> {
        let signer = self.client.read_chan_signer(reader)?;
        Ok(DynSigner::new(signer))
    }

    fn get_destination_script(&self) -> Result<Script, ()> {
        self.client.get_destination_script()
    }

    fn get_shutdown_scriptpubkey(
        &self,
    ) -> Result<lightning_signer::lightning::ln::script::ShutdownScript, ()> {
        self.client.get_shutdown_scriptpubkey()
    }
}

impl EntropySource for LampoKeysManager {
    fn get_secure_random_bytes(&self) -> [u8; 32] {
        self.client.get_secure_random_bytes()
    }
}

impl NodeSigner for LampoKeysManager {
    fn get_inbound_payment_key_material(&self) -> lightning_signer::lightning::sign::KeyMaterial {
        self.client.get_inbound_payment_key_material()
    }

    fn get_node_id(
        &self,
        recipient: lightning_signer::lightning::sign::Recipient,
    ) -> Result<lightning_signer::bitcoin::secp256k1::PublicKey, ()> {
        self.client.get_node_id(recipient)
    }

    fn ecdh(
        &self,
        recipient: lightning_signer::lightning::sign::Recipient,
        other_key: &lightning_signer::bitcoin::secp256k1::PublicKey,
        tweak: Option<&lightning_signer::bitcoin::secp256k1::Scalar>,
    ) -> Result<lightning_signer::bitcoin::secp256k1::ecdh::SharedSecret, ()> {
        self.client.ecdh(recipient, other_key, tweak)
    }

    fn sign_invoice(
        &self,
        hrp_bytes: &[u8],
        invoice_data: &[lightning_signer::bitcoin::bech32::u5],
        recipient: lightning_signer::lightning::sign::Recipient,
    ) -> Result<lightning_signer::bitcoin::secp256k1::ecdsa::RecoverableSignature, ()> {
        self.client.sign_invoice(hrp_bytes, invoice_data, recipient)
    }

    fn sign_bolt12_invoice_request(
        &self,
        invoice_request: &UnsignedInvoiceRequest,
    ) -> Result<Signature, ()> {
        self.client.sign_bolt12_invoice_request(invoice_request)
    }

    fn sign_bolt12_invoice(&self, invoice: &UnsignedBolt12Invoice) -> Result<Signature, ()> {
        self.client.sign_bolt12_invoice(invoice)
    }

    fn sign_gossip_message(
        &self,
        msg: lightning_signer::lightning::ln::msgs::UnsignedGossipMessage,
    ) -> Result<lightning_signer::bitcoin::secp256k1::ecdsa::Signature, ()> {
        self.client.sign_gossip_message(msg)
    }
}

impl SpendableKeysInterface for LampoKeysManager {
    fn spend_spendable_outputs(
        &self,
        descriptors: &[&SpendableOutputDescriptor],
        outputs: Vec<TxOut>,
        change_destination_script: Script,
        feerate_sat_per_1000_weight: u32,
        _secp_ctx: &Secp256k1<All>,
    ) -> anyhow::Result<Transaction> {
        info!("ENTER spend_spendable_outputs");
        let mut tx = create_spending_transaction(
            descriptors,
            outputs,
            Box::new(change_destination_script),
            feerate_sat_per_1000_weight,
        )?;
        let witnesses = self.client.sign_onchain_tx(&tx, descriptors);
        for (idx, w) in witnesses.into_iter().enumerate() {
            tx.input[idx].witness = Witness::from_vec(w);
        }
        Ok(tx)
    }

    fn get_sweep_address(&self) -> Address {
        self.sweep_address.clone()
    }
}
