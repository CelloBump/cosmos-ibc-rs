//! Protocol logic specific to ICS4 messages of type `MsgChannelCloseInit`.
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::ClientStateValidation;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::CloseInit;
use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use crate::core::ics24_host::path::ChannelEndPath;
use crate::core::router::Module;
use crate::core::{ContextError, ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn chan_close_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    module.on_chan_close_init_validate(&msg.port_id_on_a, &msg.chan_id_on_a)?;

    Ok(())
}

pub(crate) fn chan_close_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let extras = module.on_chan_close_init_execute(&msg.port_id_on_a, &msg.chan_id_on_a)?;
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();
            chan_end_on_a.set_state(State::Closed);
            chan_end_on_a
        };

        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel close init".to_string())?;

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let chan_id_on_b = chan_end_on_a
                .counterparty()
                .channel_id
                .clone()
                .ok_or(ContextError::ChannelError(ChannelError::Other {
                description:
                    "internal error: ChannelEnd doesn't have a counterparty channel id in CloseInit"
                        .to_string(),
            }))?;
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::CloseInitChannel(CloseInit::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                chan_id_on_b,
                conn_id_on_a,
            ))
        };
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_a.emit_ibc_event(core_event)?;

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event))?;
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelCloseInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Validate that the channel end is in a state where it can be closed.
    chan_end_on_a.verify_not_closed()?;

    // An OPEN IBC connection running on the local (host) chain should exist.
    chan_end_on_a.verify_connection_hops_length()?;

    let conn_end_on_a = ctx_a.connection_end(&chan_end_on_a.connection_hops()[0])?;

    conn_end_on_a.verify_state_matches(&ConnectionState::Open)?;

    let client_id_on_a = conn_end_on_a.client_id();
    let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
    {
        let status =
            client_state_of_b_on_a.status(ctx_a.get_client_validation_context(), client_id_on_a)?;
        if !status.is_active() {
            return Err(ClientError::ClientNotActive { status }.into());
        }
    }

    Ok(())
}
