// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::serializer::{
    ExecutionCorrectnessInput, SerializerClient, SerializerService, TSerializerClient,
};
use executor::Executor;
use executor_types::Error;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_logger::warn;
use libra_secure_net::{Error as NetError, NetworkClient, NetworkServer};
use libra_vm::LibraVM;
use std::net::SocketAddr;
use storage_client::StorageClient;

pub trait RemoteService {
    fn client(&self) -> SerializerClient {
        let network_client = NetworkClient::new(self.server_address());
        let service = Box::new(RemoteClient::new(network_client));
        SerializerClient::new_client(service)
    }

    fn server_address(&self) -> SocketAddr;
}

pub fn execute(
    storage_addr: SocketAddr,
    listen_addr: SocketAddr,
    prikey: Option<Ed25519PrivateKey>,
) {
    let block_executor = Box::new(Executor::<LibraVM>::new(
        StorageClient::new(&storage_addr).into(),
    ));
    let mut serializer_service = SerializerService::new(block_executor, prikey);
    let mut network_server = NetworkServer::new(listen_addr);

    loop {
        if let Err(e) = process_one_message(&mut network_server, &mut serializer_service) {
            warn!("Warning: Failed to process message: {}", e);
        }
    }
}

fn process_one_message(
    network_server: &mut NetworkServer,
    serializer_service: &mut SerializerService,
) -> Result<(), Error> {
    let request = network_server.read()?;
    let response = serializer_service.handle_message(request)?;
    network_server.write(&response)?;
    Ok(())
}

struct RemoteClient {
    network_client: NetworkClient,
}

impl RemoteClient {
    pub fn new(network_client: NetworkClient) -> Self {
        Self { network_client }
    }
}

impl TSerializerClient for RemoteClient {
    fn request(&mut self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        let result = loop {
            while self.network_client.write(&input_message).is_err() {}
            let res = self.network_client.read();
            match res {
                Ok(res) => break res,
                Err(NetError::RemoteStreamClosed) => (),
                Err(err) => return Err(err.into()),
            }
        };
        Ok(result)
    }
}
