use std::sync::{Arc, Mutex};

use subtle_encoding::bech32;
use tendermint::{block, consensus, evidence, public_key::Algorithm};

use crate::applications::transfer::context::{
    cosmos_adr028_escrow_address, BankKeeper, Ics20Context, TokenTransferKeeper,
    TokenTransferReader,
};
use crate::applications::transfer::{error::Error as Ics20Error, PrefixedCoin};
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::Error as Ics02Error;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::error::Error as Ics03Error;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::commitment::PacketCommitment;
use crate::core::ics04_channel::context::SendPacketReader;
use crate::core::ics04_channel::error::Error;
use crate::core::ics04_channel::handler::ModuleExtras;
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics04_channel::Version;
use crate::core::ics05_port::context::PortReader;
use crate::core::ics05_port::error::Error as PortError;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics26_routing::context::{Module, ModuleId};
use crate::mock::context::MockIbcStore;
use crate::prelude::*;
use crate::signer::Signer;
use crate::Height;

// Needed in mocks.
pub fn default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1,
            time_iota_ms: 1000,
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(core::time::Duration::new(48 * 3600, 0)),
            max_bytes: 0,
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519],
        },
        version: Some(consensus::params::VersionParams::default()),
    }
}

pub fn get_dummy_proof() -> Vec<u8> {
    "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy"
        .as_bytes()
        .to_vec()
}

pub fn get_dummy_account_id() -> Signer {
    "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
}

pub fn get_dummy_bech32_account() -> String {
    "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng".to_string()
}

pub fn get_dummy_transfer_module() -> DummyTransferModule {
    let ibc_store = Arc::new(Mutex::new(MockIbcStore::default()));
    DummyTransferModule { ibc_store }
}
#[derive(Debug)]
pub struct DummyTransferModule {
    ibc_store: Arc<Mutex<MockIbcStore>>,
}

impl DummyTransferModule {
    pub fn new(ibc_store: Arc<Mutex<MockIbcStore>>) -> Self {
        Self { ibc_store }
    }
}

impl Module for DummyTransferModule {
    fn on_chan_open_init(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), Error> {
        Ok((
            ModuleExtras {
                events: Vec::new(),
                log: Vec::new(),
            },
            version.clone(),
        ))
    }

    fn on_chan_open_try(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), Error> {
        Ok((
            ModuleExtras {
                events: Vec::new(),
                log: Vec::new(),
            },
            counterparty_version.clone(),
        ))
    }
}

impl TokenTransferKeeper for DummyTransferModule {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), Error> {
        self.ibc_store
            .lock()
            .unwrap()
            .packet_commitment
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, commitment);
        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Error> {
        self.ibc_store
            .lock()
            .unwrap()
            .next_sequence_send
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }
}

impl PortReader for DummyTransferModule {
    fn lookup_module_by_port(&self, _port_id: &PortId) -> Result<ModuleId, PortError> {
        unimplemented!()
    }
}

impl BankKeeper for DummyTransferModule {
    type AccountId = Signer;

    fn send_coins(
        &mut self,
        _from: &Self::AccountId,
        _to: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    fn mint_coins(
        &mut self,
        _account: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    fn burn_coins(
        &mut self,
        _account: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }
}


impl TokenTransferReader for DummyTransferModule {
    type AccountId = Signer;

    fn get_port(&self) -> Result<PortId, Ics20Error> {
        Ok(PortId::transfer())
    }

    fn get_channel_escrow_address(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<<Self as TokenTransferReader>::AccountId, Ics20Error> {
        let addr = cosmos_adr028_escrow_address(port_id, channel_id);
        Ok(bech32::encode("cosmos", addr).parse().unwrap())
    }

    fn is_send_enabled(&self) -> bool {
        true
    }

    fn is_receive_enabled(&self) -> bool {
        true
    }
}

impl SendPacketReader for DummyTransferModule {
    fn channel_end(&self, port_id: &PortId, channel_id: &ChannelId) -> Result<ChannelEnd, Error> {
        match self
            .ibc_store
            .lock()
            .unwrap()
            .channels
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(channel_end) => Ok(channel_end.clone()),
            None => Err(Error::channel_not_found(
                port_id.clone(),
                channel_id.clone(),
            )),
        }
    }

    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, Error> {
        match self.ibc_store.lock().unwrap().connections.get(cid) {
            Some(connection_end) => Ok(connection_end.clone()),
            None => Err(Ics03Error::connection_not_found(cid.clone())),
        }
        .map_err(Error::ics03_connection)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Error> {
        match self.ibc_store.lock().unwrap().clients.get(client_id) {
            Some(client_record) => client_record
                .client_state
                .clone()
                .ok_or_else(|| Ics02Error::client_not_found(client_id.clone())),
            None => Err(Ics02Error::client_not_found(client_id.clone())),
        }
        .map_err(|e| Error::ics03_connection(Ics03Error::ics02_client(e)))
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Error> {
        match self.ibc_store.lock().unwrap().clients.get(client_id) {
            Some(client_record) => match client_record.consensus_states.get(&height) {
                Some(consensus_state) => Ok(consensus_state.clone()),
                None => Err(Ics02Error::consensus_state_not_found(
                    client_id.clone(),
                    height,
                )),
            },
            None => Err(Ics02Error::consensus_state_not_found(
                client_id.clone(),
                height,
            )),
        }
        .map_err(|e| Error::ics03_connection(Ics03Error::ics02_client(e)))
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Error> {
        match self
            .ibc_store
            .lock()
            .unwrap()
            .next_sequence_send
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(Error::missing_next_send_seq(
                port_id.clone(),
                channel_id.clone(),
            )),
        }
    }

    fn hash(&self, value: Vec<u8>) -> Vec<u8> {
        use sha2::Digest;

        sha2::Sha256::digest(value).to_vec()
    }
}

impl TransferContext for DummyTransferModule {
    type AccountId = Signer;
}
