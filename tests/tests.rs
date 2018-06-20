extern crate bitcoin;
extern crate exonum;
extern crate exonum_btc_anchoring;
extern crate exonum_testkit;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate btc_transaction_utils;

use exonum_btc_anchoring::{config::GlobalConfig, rpc::BtcRelay, test_data::AnchoringTestKit,
                           BTC_ANCHORING_SERVICE_NAME};

use exonum::helpers::Height;

#[cfg(feature = "rpc_tests")]
#[test]
fn simple() {
    let validators_num = 4;
    let mut anchoring_testkit = AnchoringTestKit::new_with_testnet(validators_num, 70000, 4);

    assert!(anchoring_testkit.last_anchoring_tx().is_none());

    let signatures = anchoring_testkit.create_signature_tx_for_validators(2);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(4));

    let tx0 = anchoring_testkit.last_anchoring_tx();
    assert!(tx0.is_some());
    let tx0 = tx0.unwrap();
    let tx0_meta = tx0.anchoring_metadata().unwrap();
    assert!(tx0_meta.1.block_height == Height(0));

    let signatures = anchoring_testkit.create_signature_tx_for_validators(2);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(8));

    let tx1 = anchoring_testkit.last_anchoring_tx();
    assert!(tx1.is_some());

    let tx1 = tx1.unwrap();
    let tx1_meta = tx1.anchoring_metadata().unwrap();

    assert!(tx0.id() == tx1.prev_tx_id());

    // script_pubkey should be the same
    assert!(tx0_meta.0 == tx1_meta.0);
    assert!(tx1_meta.1.block_height == Height(4));
}

#[cfg(feature = "rpc_tests")]
#[test]
fn additional_funding() {
    let validators_num = 4;
    let initial_sum = 50000;
    let mut anchoring_testkit = AnchoringTestKit::new_with_testnet(validators_num, initial_sum, 4);

    let signatures = anchoring_testkit.create_signature_tx_for_validators(2);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(4));

    let tx0 = anchoring_testkit.last_anchoring_tx();
    assert!(tx0.is_some());
    let tx0 = tx0.unwrap();
    assert!(tx0.0.input.len() == 1);

    let output_val0 = tx0.0.output.iter().map(|x| x.value).max().unwrap();
    assert!(output_val0 < initial_sum);

    //creating new funding tx"
    let rpc_client = anchoring_testkit.rpc_client();
    let address = anchoring_testkit.anchoring_address();

    let new_funding_tx = rpc_client.send_to_address(&address, initial_sum).unwrap();
    let mut configuration_change_proposal = anchoring_testkit.configuration_change_proposal();
    let service_configuration = GlobalConfig {
        funding_transaction: Some(new_funding_tx),
        ..configuration_change_proposal.service_config(BTC_ANCHORING_SERVICE_NAME)
    };

    configuration_change_proposal
        .set_service_config(BTC_ANCHORING_SERVICE_NAME, service_configuration);
    configuration_change_proposal.set_actual_from(Height(6));
    anchoring_testkit.commit_configuration_change(configuration_change_proposal);
    anchoring_testkit.create_blocks_until(Height(6));

    let signatures = anchoring_testkit.create_signature_tx_for_validators(2);
    anchoring_testkit.create_block_with_transactions(signatures);

    let tx1 = anchoring_testkit.last_anchoring_tx().unwrap();
    let tx1_meta = tx1.anchoring_metadata().unwrap();
    assert!(tx1_meta.1.block_height == Height(4));

    assert!(tx1.0.input.len() == 2);

    let output_val1 = tx1.0.output.iter().map(|x| x.value).max().unwrap();
    assert!(output_val1 > output_val0);
    assert!(output_val1 > initial_sum);
}

#[cfg(feature = "rpc_tests")]
#[test]
fn address_changed() {
    let validators_num = 5;
    let mut anchoring_testkit = AnchoringTestKit::new_with_testnet(validators_num, 150000, 4);
    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(4));

    let tx0 = anchoring_testkit.last_anchoring_tx();
    assert!(tx0.is_some());
    let tx0 = tx0.unwrap();
    let tx0_meta = tx0.anchoring_metadata().unwrap();

    let signatures = anchoring_testkit.create_signature_tx_for_validators(4);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(6));

    // removing one of validators
    let mut configuration_change_proposal = anchoring_testkit.configuration_change_proposal();
    let mut validators = configuration_change_proposal.validators().to_vec();

    let _ = validators.pop().unwrap();
    configuration_change_proposal.set_validators(validators);

    let config: GlobalConfig =
        configuration_change_proposal.service_config(BTC_ANCHORING_SERVICE_NAME);

    let mut keys = config.public_keys.clone();
    let _ = keys.pop().unwrap();

    let service_configuration = GlobalConfig {
        public_keys: keys,
        ..config
    };
    configuration_change_proposal
        .set_service_config(BTC_ANCHORING_SERVICE_NAME, service_configuration);
    configuration_change_proposal.set_actual_from(Height(16));
    anchoring_testkit.commit_configuration_change(configuration_change_proposal);
    anchoring_testkit.create_blocks_until(Height(7));

    anchoring_testkit.renew_address();
    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(10));

    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(12));

    let tx_transition = anchoring_testkit.last_anchoring_tx().unwrap();

    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(16));

    let tx_same = anchoring_testkit.last_anchoring_tx().unwrap();
    // anchoring is paused till new config
    assert!(tx_transition == tx_same);

    anchoring_testkit.create_blocks_until(Height(17));
    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);

    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(20));

    let txchanged = anchoring_testkit.last_anchoring_tx().unwrap();
    let txchanged_meta = txchanged.anchoring_metadata().unwrap();

    assert!(tx_transition != txchanged);
    // script_pubkey should *not* be the same
    assert!(tx0_meta.0 != txchanged_meta.0);
}

#[cfg(feature = "rpc_tests")]
#[test]
fn address_changed_and_new_funding_tx() {
    let validators_num = 5;
    let initial_sum = 150000;
    let mut anchoring_testkit = AnchoringTestKit::new_with_testnet(validators_num, initial_sum, 4);
    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(4));

    let tx0 = anchoring_testkit.last_anchoring_tx();
    assert!(tx0.is_some());
    let tx0 = tx0.unwrap();
    let tx0_meta = tx0.anchoring_metadata().unwrap();
    let output_val0 = tx0.0.output.iter().map(|x| x.value).max().unwrap();

    // removing one of validators
    let mut configuration_change_proposal = anchoring_testkit.configuration_change_proposal();
    let mut validators = configuration_change_proposal.validators().to_vec();

    let _ = validators.pop().unwrap();
    configuration_change_proposal.set_validators(validators);

    let config: GlobalConfig =
        configuration_change_proposal.service_config(BTC_ANCHORING_SERVICE_NAME);

    let mut keys = config.public_keys.clone();
    let _ = keys.pop().unwrap();

    let mut service_configuration = GlobalConfig {
        public_keys: keys,
        ..config
    };

    // additional funding
    let rpc_client = anchoring_testkit.rpc_client();
    let new_address = service_configuration.anchoring_address();

    let new_funding_tx = rpc_client
        .send_to_address(&new_address, initial_sum)
        .unwrap();

    service_configuration.funding_transaction = Some(new_funding_tx);

    configuration_change_proposal
        .set_service_config(BTC_ANCHORING_SERVICE_NAME, service_configuration);
    configuration_change_proposal.set_actual_from(Height(16));
    anchoring_testkit.commit_configuration_change(configuration_change_proposal);

    anchoring_testkit.create_blocks_until(Height(7));

    anchoring_testkit.renew_address();

    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(10));

    let tx_transition = anchoring_testkit.last_anchoring_tx().unwrap();

    //new funding transaction should not be consumed during creation of transition tx
    assert!(tx_transition.0.input.len() == 1);

    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);
    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(16));

    anchoring_testkit.create_blocks_until(Height(17));
    let signatures = anchoring_testkit.create_signature_tx_for_validators(3);

    anchoring_testkit.create_block_with_transactions(signatures);
    anchoring_testkit.create_blocks_until(Height(20));

    let txchanged = anchoring_testkit.last_anchoring_tx().unwrap();
    let txchanged_meta = txchanged.anchoring_metadata().unwrap();
    let output_changed = txchanged.0.output.iter().map(|x| x.value).max().unwrap();

    assert!(tx_transition != txchanged);
    assert!(txchanged.0.input.len() == 2);
    assert!(txchanged.0.input.len() == 2);

    // script_pubkey should *not* be the same
    assert!(tx0_meta.0 != txchanged_meta.0);

    assert!(output_changed > output_val0);
    assert!(output_changed > initial_sum);
}