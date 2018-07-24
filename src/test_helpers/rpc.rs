// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bitcoin::util::address::Address;

use std::collections::VecDeque;

use exonum::crypto::Hash;

use btc;
use failure;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde_json::value::{from_value, to_value, Value};

use bitcoin_rpc;

use exonum::encoding::serialize::FromHex;

use rpc::{BitcoinRpcConfig, BtcRelay, TransactionInfo as BtcTransactionInfo};

pub enum FakeRelayRequest {
    SendToAddress {
        addr: &Address,
        satoshis: u64,
        response: Result<btc::Transaction, failure::Error>,
    },
    TransactionInfo {
        id: &Hash,
        response: Result<Option<BtcTransactionInfo>, failure::Error>,
    },
    SendTransaction {
        transaction: &btc::Transaction,
        response: Result<Hash, failure::Error>,
    },
    WatchAddress {
        addr: &Address,
        rescan: bool,
        response: Result<(), failure::Error>,
    },
}

struct FakeBtcRelay;

impl BtcRelay for FakeBtcRelay {
    fn send_to_address(
        &self,
        addr: &Address,
        satoshis: u64,
    ) -> Result<btc::Transaction, failure::Error> {

    }

    fn transaction_info(&self, id: &Hash) -> Result<Option<BtcTransactionInfo>, failure::Error> {}

    fn send_transaction(&self, transaction: &btc::Transaction) -> Result<Hash, failure::Error> {}

    fn watch_address(&self, addr: &Address, rescan: bool) -> Result<(), failure::Error> {}

    fn config(&self) -> BitcoinRpcConfig {
        self.rpc.clone()
    }
}
