//! Lampo Offchain manager.
//!
//! The offchain manager will manage all the necessary
//! information about the lightning network operation.
//!
//! Such as generate and invoice or pay an invoice.
//!
//! This module will also be able to interact with
//! other feature like onion message, and more general
//! with the network graph. But this is not so clear yet.
//!
//! Author: Vincenzo Palazzo <vincenzopalazzo@member.fsf.org>
use std::sync::Arc;
use std::time::Duration;

use lampo_common::conf::LampoConf;
use lampo_common::error;
use lampo_common::keymanager::KeysManager;
use lampo_common::ldk;
use lampo_common::ldk::ln::channelmanager::Retry;

use crate::utils::logger::LampoLogger;

use super::LampoChannelManager;

pub struct OffchainManager {
    channel_manager: Arc<LampoChannelManager>,
    keys_manager: Arc<KeysManager>,
    logger: Arc<LampoLogger>,
    lampo_conf: Arc<LampoConf>,
}

impl OffchainManager {
    // FIXME: use the build pattern here
    pub fn new(
        keys_manager: Arc<KeysManager>,
        channel_manager: Arc<LampoChannelManager>,
        logger: Arc<LampoLogger>,
        lampo_conf: Arc<LampoConf>,
    ) -> error::Result<Self> {
        Ok(Self {
            channel_manager,
            keys_manager,
            logger,
            lampo_conf,
        })
    }

    /// Generate an invoice with a specific amount and a specific
    /// description.
    pub fn generate_invoice(
        &self,
        amount_msat: Option<u64>,
        description: &str,
        expiring_in: u32,
    ) -> error::Result<ldk::invoice::Bolt11Invoice> {
        let currency = ldk::invoice::Currency::try_from(self.lampo_conf.network)?;
        let invoice = ldk::invoice::utils::create_invoice_from_channelmanager(
            &self.channel_manager.manager(),
            self.keys_manager.clone(),
            self.logger.clone(),
            currency,
            amount_msat,
            description.to_string(),
            expiring_in,
            None,
            // FIXME: improve the error inside the ldk side
        )
        .map_err(|err| error::anyhow!(err))?;
        Ok(invoice)
    }

    pub fn decode_invoice(&self, invoice_str: &str) -> error::Result<ldk::invoice::Bolt11Invoice> {
        let invoice = invoice_str.parse::<ldk::invoice::Bolt11Invoice>()?;
        Ok(invoice)
    }

    pub fn pay_invoice(&self, invoice_str: &str, amount_msat: Option<u64>) -> error::Result<()> {
        let invoice = self.decode_invoice(invoice_str)?;
        if invoice.amount_milli_satoshis().is_none() {
            ldk::invoice::payment::pay_zero_value_invoice(
                &invoice,
                amount_msat.ok_or(error::anyhow!(
                    "invoice with no amount, and amount must be specified"
                ))?,
                Retry::Timeout(Duration::from_secs(10)),
                &self.channel_manager.manager(),
            )
            .map_err(|err| error::anyhow!("{:?}", err))?;
        } else {
            ldk::invoice::payment::pay_invoice(
                &invoice,
                Retry::Timeout(Duration::from_secs(10)),
                &self.channel_manager.manager(),
            )
            .map_err(|err| error::anyhow!("{:?}", err))?;
        }
        Ok(())
    }

    pub fn keysend(&self) -> error::Result<()> {
        todo!()
    }
}