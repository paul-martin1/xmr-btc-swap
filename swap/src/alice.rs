//! Run an XMR/BTC swap in the role of Alice.
//! Alice holds XMR and wishes receive BTC.
use anyhow::Result;
use libp2p::{
    core::{identity::Keypair, Multiaddr},
    request_response::ResponseChannel,
    NetworkBehaviour, PeerId,
};
use std::{thread, time::Duration};
use tracing::{debug, warn};

mod messenger;

use self::messenger::*;
use crate::{
    monero,
    network::{
        peer_tracker::{self, PeerTracker},
        request_response::{AliceToBob, TIMEOUT},
        transport, TokioExecutor,
    },
    Never, SwapParams,
};

pub type Swarm = libp2p::Swarm<Alice>;

pub async fn swap(listen: Multiaddr) -> Result<()> {
    let mut swarm = new_swarm(listen)?;

    match swarm.next().await {
        BehaviourOutEvent::Request(messenger::BehaviourOutEvent::Btc { btc, channel }) => {
            debug!("Got request from Bob");
            let params = SwapParams {
                btc,
                // TODO: Do a real calculation.
                xmr: monero::Amount::from_piconero(10),
            };

            let msg = AliceToBob::Amounts(params);
            swarm.send(channel, msg);
        }
        other => panic!("unexpected event: {:?}", other),
    }

    warn!("parking thread ...");
    thread::park();
    Ok(())
}

fn new_swarm(listen: Multiaddr) -> Result<Swarm> {
    use anyhow::Context as _;

    let behaviour = Alice::default();

    let local_key_pair = behaviour.identity();
    let local_peer_id = behaviour.peer_id();

    let transport = transport::build(local_key_pair)?;

    let mut swarm = libp2p::swarm::SwarmBuilder::new(transport, behaviour, local_peer_id.clone())
        .executor(Box::new(TokioExecutor {
            handle: tokio::runtime::Handle::current(),
        }))
        .build();

    Swarm::listen_on(&mut swarm, listen.clone())
        .with_context(|| format!("Address is not supported: {:#}", listen))?;

    tracing::info!("Initialized swarm: {}", local_peer_id);

    Ok(swarm)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum BehaviourOutEvent {
    Request(messenger::BehaviourOutEvent),
    ConnectionEstablished(PeerId),
    Never, // FIXME: Why do we need this?
}

impl From<Never> for BehaviourOutEvent {
    fn from(_: Never) -> Self {
        BehaviourOutEvent::Never
    }
}

impl From<messenger::BehaviourOutEvent> for BehaviourOutEvent {
    fn from(event: messenger::BehaviourOutEvent) -> Self {
        BehaviourOutEvent::Request(event)
    }
}

impl From<peer_tracker::BehaviourOutEvent> for BehaviourOutEvent {
    fn from(event: peer_tracker::BehaviourOutEvent) -> Self {
        match event {
            peer_tracker::BehaviourOutEvent::ConnectionEstablished(id) => {
                BehaviourOutEvent::ConnectionEstablished(id)
            }
        }
    }
}

/// A `NetworkBehaviour` that represents an XMR/BTC swap node as Alice.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourOutEvent", event_process = false)]
#[allow(missing_debug_implementations)]
pub struct Alice {
    net: Messenger,
    pt: PeerTracker,
    #[behaviour(ignore)]
    identity: Keypair,
}

impl Alice {
    pub fn identity(&self) -> Keypair {
        self.identity.clone()
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId::from(self.identity.public())
    }

    /// Alice always sends her messages as a response to a request from Bob.
    pub fn send(&mut self, channel: ResponseChannel<AliceToBob>, msg: AliceToBob) {
        self.net.send(channel, msg);
    }
}

impl Default for Alice {
    fn default() -> Self {
        let identity = Keypair::generate_ed25519();
        let timeout = Duration::from_secs(TIMEOUT);

        Self {
            net: Messenger::new(timeout),
            pt: PeerTracker::default(),
            identity,
        }
    }
}
