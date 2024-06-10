use anyhow::anyhow;
use anyhow::Result;
use lightning_signer::bitcoin::{Script, Transaction, TxIn, TxOut, Witness};
use lightning_signer::lightning::sign::{
    DelayedPaymentOutputDescriptor, SpendableOutputDescriptor, StaticPaymentOutputDescriptor,
};
use lightning_signer::{
    bitcoin::PackedLockTime,
    util::transaction_utils::{maybe_add_change_output, MAX_VALUE_MSAT},
};
use std::collections::HashSet;
use vls_proxy::vls_protocol_signer::vls_protocol::serde_bolt::bitcoin::Sequence;

/// Copied directly from `lnrod`
pub(crate) fn create_spending_transaction(
    descriptors: &[&SpendableOutputDescriptor],
    outputs: Vec<TxOut>,
    change_destination_script: Box<Script>,
    feerate_sats_per_1000_weight: u32,
) -> Result<Transaction> {
    let mut input = Vec::new();
    let mut input_value = 0;
    let mut witness_weight = 0;
    let mut output_set = HashSet::with_capacity(descriptors.len());
    for outp in descriptors {
        match outp {
            SpendableOutputDescriptor::StaticPaymentOutput(descriptor) => {
                input.push(TxIn {
                    previous_output: descriptor.outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: Sequence::ZERO,
                    witness: Witness::new(),
                });
                witness_weight += StaticPaymentOutputDescriptor::max_witness_length(descriptor);
                input_value += descriptor.output.value;
                if !output_set.insert(descriptor.outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }

            SpendableOutputDescriptor::DelayedPaymentOutput(descriptor) => {
                input.push(TxIn {
                    previous_output: descriptor.outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: Sequence(descriptor.to_self_delay as u32),
                    witness: Witness::default(),
                });
                witness_weight += DelayedPaymentOutputDescriptor::MAX_WITNESS_LENGTH;
                input_value += descriptor.output.value;
                if !output_set.insert(descriptor.outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }

            SpendableOutputDescriptor::StaticOutput {
                ref outpoint,
                ref output,
            } => {
                input.push(TxIn {
                    previous_output: outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: Sequence::ZERO,
                    witness: Witness::default(),
                });
                witness_weight += 1 + 73 + 34;
                input_value += output.value;
                if !output_set.insert(*outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }
        }

        if input_value > MAX_VALUE_MSAT / 1000 {
            return Err(anyhow!("overflow"));
        }
    }

    let mut spend_tx = Transaction {
        version: 2,
        lock_time: PackedLockTime(0),
        input,
        output: outputs,
    };
    maybe_add_change_output(
        &mut spend_tx,
        input_value,
        witness_weight,
        feerate_sats_per_1000_weight,
        *change_destination_script,
    )
    .map_err(|()| anyhow!("could not add or change"))?;
    Ok(spend_tx)
}
