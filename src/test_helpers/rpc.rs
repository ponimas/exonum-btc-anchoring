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

#[derive(Debug)]
pub enum FakeRelayRequest {
    SendToAddress { addr: Address, satoshis: u64 },
    TransactionInfo { id: Hash },
    SendTransaction { transaction: btc::Transaction },
    WatchAddress { addr: Address, rescan: bool },
}

#[derive(Debug)]
pub enum FakeRelayResponse {
    SendToAddress(Result<btc::Transaction, failure::Error>),
    TransactionInfo(Result<Option<BtcTransactionInfo>, failure::Error>),
    SendTransaction(Result<Hash, failure::Error>),
    WatchAddress(Result<(), failure::Error>),
}

#[derive(Debug, Clone, Default)]
pub struct TestRequests(Arc<Mutex<VecDeque<(FakeRelayRequest, FakeRelayResponse)>>>);

impl TestRequests {
    pub fn new() -> TestRequests {
        TestRequests(Arc::new(Mutex::new(VecDeque::new())))
    }

    pub fn expect<I: IntoIterator<Item = (FakeRelayRequest, FakeRelayResponse)>>(
        &self,
        requests: I,
    ) {
        self.0.lock().unwrap().extend(requests);
    }
}

// -
// -#[derive(Debug)]
// -pub struct FakeBitcoinRpcClient {
// -    pub requests: TestRequests,
// -    rpc: BitcoinRpcConfig,
// -}
// -
// -impl FakeBitcoinRpcClient {
// -    pub fn new() -> Self {
// -        Self {
// -            requests: TestRequests::new(),
// -            rpc: BitcoinRpcConfig {
// -                host: String::from("http://127.0.0.1:1234"),
// -                username: None,
// -                password: None,
// -            },
// -        }
// -    }
// -
// -    fn request<T, P>(&self, method: &str, params: P) -> bitcoin_rpc::Result<T>
// -    where
// -        T: ::std::fmt::Debug,
// -        P: AsRef<bitcoin_rpc::Params>,
// -        for<'de> T: Deserialize<'de>,
// -    {
// -        let params = params.as_ref();
// -        let expected = self.requests.0.lock().unwrap().pop_front().expect(
// -            format!(
// -                "expected response for method={}, \
// -                 params={:#?}",
// -                method, params
// -            ).as_str(),
// -        );
// -
// -        assert_eq!(expected.method, method);
// -        assert_eq!(
// -            &expected.params, params,
// -            "Invalid params for method {}!",
// -            method
// -        );
// -
// -        let response = expected.response?;
// -        trace!(
// -            "method: {}, params={:?}, response={:#}",
// -            method,
// -            params,
// -            response
// -        );
// -        from_value(response).map_err(|e| bitcoin_rpc::Error::Rpc(bitcoin_rpc::RpcError::Json(e)))
// -    }
#[derive(Debug, Default)]
pub struct FakeBtcRelay {
    pub requests: TestRequests,
    rpc: BitcoinRpcConfig,
}

impl FakeBtcRelay {
    fn request(&self, request: FakeRelayRequest) -> FakeRelayResponse {
        let (expected_request, response) = self.requests
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect(format!("expected request {:?}", request).as_str());

        assert_matches!(request, expected_request);

        trace!("request: {:?}, response={:?}", expected_request, response);
        response
    }
}

impl BtcRelay for FakeBtcRelay {
    fn send_to_address(
        &self,
        addr: &Address,
        satoshis: u64,
    ) -> Result<btc::Transaction, failure::Error> {
        if let FakeRelayResponse::SendToAddress(r) = self.request(FakeRelayRequest::SendToAddress {
            addr: addr.clone(),
            satoshis,
        }) {
            r
        } else {
            panic!();
        }
    }

    fn transaction_info(&self, id: &Hash) -> Result<Option<BtcTransactionInfo>, failure::Error> {
        if let FakeRelayResponse::TransactionInfo(r) =
            self.request(FakeRelayRequest::TransactionInfo { id: *id })
        {
            r
        } else {
            panic!();
        }
    }

    fn send_transaction(&self, transaction: &btc::Transaction) -> Result<Hash, failure::Error> {
        if let FakeRelayResponse::SendTransaction(r) =
            self.request(FakeRelayRequest::SendTransaction {
                transaction: transaction.clone(),
            }) {
            r
        } else {
            panic!();
        }
    }

    fn watch_address(&self, addr: &Address, rescan: bool) -> Result<(), failure::Error> {
        if let FakeRelayResponse::WatchAddress(r) = self.request(FakeRelayRequest::WatchAddress {
            addr: addr.clone(),
            rescan,
        }) {
            r
        } else {
            panic!();
        }
    }

    fn config(&self) -> BitcoinRpcConfig {
        self.rpc.clone()
    }
}
