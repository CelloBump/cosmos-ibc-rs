//! Defines the client message type for the ICS-08 Wasm light client.

use ibc_primitives::proto::Protobuf;
use ibc_proto::ibc::lightclients::wasm::v1::ClientMessage as RawClientMessage;

use crate::Bytes;

pub const WASM_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientMessage";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMessage {
    pub data: Bytes,
}

impl Protobuf<RawClientMessage> for ClientMessage {}

impl From<RawClientMessage> for ClientMessage {
    fn from(raw: RawClientMessage) -> Self {
        Self { data: raw.data }
    }
}

impl From<ClientMessage> for RawClientMessage {
    fn from(value: ClientMessage) -> Self {
        RawClientMessage { data: value.data }
    }
}
