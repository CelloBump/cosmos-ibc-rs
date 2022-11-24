use crate::core::ics02_client::error as client_error;
use crate::core::ics03_connection::version::Version;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::proofs::ProofError;
use crate::signer::SignerError;
use crate::Height;

use alloc::string::String;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// ICS02 client error
    Client(client_error::ClientError),
    /// connection state is unknown: `{state}`
    InvalidState { state: i32 },
    /// connection exists (was initialized) already: `{connection_id}`
    ConnectionExistsAlready { connection_id: ConnectionId },
    /// connection end for identifier `{connection_id}` was never initialized
    ConnectionMismatch { connection_id: ConnectionId },
    /// consensus height claimed by the client on the other party is too advanced: `{target_height}` (host chain current height: `{current_height}`)
    InvalidConsensusHeight {
        target_height: Height,
        current_height: Height,
    },
    /// consensus height claimed by the client on the other party has been pruned: `{target_height}` (host chain oldest height: `{oldest_height}`)
    StaleConsensusHeight {
        target_height: Height,
        oldest_height: Height,
    },
    /// identifier error
    InvalidIdentifier(ValidationError),
    /// ConnectionEnd domain object could not be constructed out of empty proto object
    EmptyProtoConnectionEnd,
    /// empty supported versions
    EmptyVersions,
    /// empty supported features
    EmptyFeatures,
    /// no common version
    NoCommonVersion,
    /// version \"`{version}`\" not supported
    VersionNotSupported { version: Version },
    /// invalid address
    InvalidAddress,
    /// missing proof height
    MissingProofHeight,
    /// missing consensus height
    MissingConsensusHeight,
    /// invalid connection proof
    InvalidProof(ProofError),
    /// error verifying connnection state
    VerifyConnectionState(client_error::ClientError),
    /// invalid signer
    Signer(SignerError),
    /// no connection was found for the previous connection id provided `{connection_id}`
    ConnectionNotFound { connection_id: ConnectionId },
    /// invalid counterparty
    InvalidCounterparty,
    /// counterparty chosen connection id `{connection_id}` is different than the connection id `{counterparty_connection_id}`
    ConnectionIdMismatch {
        connection_id: ConnectionId,
        counterparty_connection_id: ConnectionId,
    },
    /// missing counterparty
    MissingCounterparty,
    /// missing counterparty prefix
    MissingCounterpartyPrefix,
    /// missing client state
    MissingClientState,
    /// client proof must be present
    NullClientProof,
    /// the client id does not match any client state: `{client_id}`
    FrozenClient { client_id: ClientId },
    /// the connection proof verification failed
    ConnectionVerificationFailure,
    /// the consensus proof verification failed (height: `{height}`)
    ConsensusStateVerificationFailure {
        height: Height,
        client_error: client_error::ClientError,
    },
    /// the client state proof verification failed for client id `{client_id}`
    ClientStateVerificationFailure {
        // TODO: use more specific error source
        client_id: ClientId,
        client_error: client_error::ClientError,
    },
    /// implementation specific error
    ImplementationSpecific,
    /// invalid client state: `{reason}`
    InvalidClientState { reason: String },
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::Client(e) => Some(e),
            Error::InvalidIdentifier(e) => Some(e),
            Error::InvalidProof(e) => Some(e),
            Error::VerifyConnectionState(e) => Some(e),
            Error::Signer(e) => Some(e),
            Error::ConsensusStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            Error::ClientStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            _ => None,
        }
    }
}
