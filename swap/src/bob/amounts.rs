use anyhow::Result;
use libp2p::{
    request_response::{
        handler::RequestProtocol, ProtocolSupport, RequestId, RequestResponse,
        RequestResponseConfig, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess, PollParameters},
    NetworkBehaviour, PeerId,
};
use std::{
    collections::VecDeque,
    task::{Context, Poll},
    time::Duration,
};
use tracing::error;

use crate::{
    network::request_response::{AliceToBob, BobToAlice, Codec, Protocol},
    SwapParams,
};

#[derive(Debug)]
pub enum OutEvent {
    Amounts(SwapParams),
}

/// A `NetworkBehaviour` that represents getting the amounts of an XMR/BTC swap.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent", poll_method = "poll")]
#[allow(missing_debug_implementations)]
pub struct Amounts {
    rr: RequestResponse<Codec>,
    #[behaviour(ignore)]
    events: VecDeque<OutEvent>,
}

impl Amounts {
    pub fn new(timeout: Duration) -> Self {
        let mut config = RequestResponseConfig::default();
        config.set_request_timeout(timeout);

        Self {
            rr: RequestResponse::new(
                Codec::default(),
                vec![(Protocol, ProtocolSupport::Full)],
                config,
            ),
            events: Default::default(),
        }
    }

    pub fn request_amounts(&mut self, alice: PeerId, btc: ::bitcoin::Amount) -> Result<RequestId> {
        let msg = BobToAlice::AmountsFromBtc(btc);
        let id = self.rr.send_request(&alice, msg);

        Ok(id)
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<RequestProtocol<Codec>, OutEvent>> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(event));
        }

        Poll::Pending
    }
}

impl NetworkBehaviourEventProcess<RequestResponseEvent<BobToAlice, AliceToBob>> for Amounts {
    fn inject_event(&mut self, event: RequestResponseEvent<BobToAlice, AliceToBob>) {
        match event {
            RequestResponseEvent::Message {
                peer: _,
                message: RequestResponseMessage::Request { .. },
            } => panic!("Bob should never get a request from Alice"),
            RequestResponseEvent::Message {
                peer: _,
                message:
                    RequestResponseMessage::Response {
                        response,
                        request_id: _,
                    },
            } => match response {
                AliceToBob::Amounts(p) => self.events.push_back(OutEvent::Amounts(p)),
            },

            RequestResponseEvent::InboundFailure { .. } => {
                panic!("Bob should never get a request from Alice, so should never get an InboundFailure");
            }
            RequestResponseEvent::OutboundFailure {
                peer: _,
                request_id: _,
                error,
            } => {
                error!("Outbound failure: {:?}", error);
            }
        }
    }
}
