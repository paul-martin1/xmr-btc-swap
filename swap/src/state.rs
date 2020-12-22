use crate::SwapAmounts;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use xmr_btc::{alice, bitcoin::EncryptedSignature, bob, monero, serde::monero_private_key};

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Swap {
    Alice(Alice),
    Bob(Bob),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Alice {
    Started {
        amounts: SwapAmounts,
        state0: alice::State0,
    },
    Negotiated(alice::State3),
    BtcLocked(alice::State3),
    XmrLocked(alice::State3),
    EncSigLearned {
        state: alice::State3,
        encrypted_signature: EncryptedSignature,
    },
    CancelTimelockExpired(alice::State3),
    BtcCancelled(alice::State3),
    BtcPunishable(alice::State3),
    BtcRefunded {
        state3: alice::State3,
        #[serde(with = "monero_private_key")]
        spend_key: monero::PrivateKey,
    },
    Done(EndState),
}

#[derive(Clone, strum::Display, Debug, Deserialize, Serialize, PartialEq)]
pub enum EndState {
    SafelyAborted,
    BtcRedeemed,
    XmrRefunded,
    BtcPunished,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Bob {
    Negotiated { state2: bob::State2 },
    BtcLocked { state3: bob::State3 },
    XmrLocked { state4: bob::State4 },
    EncSigSent { state4: bob::State4 },
    BtcRedeemed(bob::State5),
    CancelTimelockExpired(bob::State4),
    BtcCancelled(bob::State4),
    SwapComplete,
}

impl From<Alice> for Swap {
    fn from(from: Alice) -> Self {
        Swap::Alice(from)
    }
}

impl From<Bob> for Swap {
    fn from(from: Bob) -> Self {
        Swap::Bob(from)
    }
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Swap::Alice(alice) => Display::fmt(alice, f),
            Swap::Bob(bob) => Display::fmt(bob, f),
        }
    }
}

impl Display for Alice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Alice::Started { .. } => write!(f, "Started"),
            Alice::Negotiated(_) => f.write_str("Handshake complete"),
            Alice::BtcLocked(_) => f.write_str("Bitcoin locked"),
            Alice::XmrLocked(_) => f.write_str("Monero locked"),
            Alice::CancelTimelockExpired(_) => f.write_str("Cancel timelock is expired"),
            Alice::BtcCancelled(_) => f.write_str("Bitcoin cancel transaction published"),
            Alice::BtcPunishable(_) => f.write_str("Bitcoin punishable"),
            Alice::BtcRefunded { .. } => f.write_str("Monero refundable"),
            Alice::Done(end_state) => write!(f, "Done: {}", end_state),
            Alice::EncSigLearned { .. } => f.write_str("Encrypted signature learned"),
        }
    }
}

impl Display for Bob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bob::Negotiated { .. } => f.write_str("Handshake complete"),
            Bob::BtcLocked { .. } => f.write_str("Bitcoin locked"),
            Bob::XmrLocked { .. } => f.write_str("Monero locked"),
            Bob::CancelTimelockExpired(_) => f.write_str("Cancel timelock is expired"),
            Bob::BtcCancelled(_) => f.write_str("Bitcoin refundable"),
            Bob::BtcRedeemed(_) => f.write_str("Monero redeemable"),
            Bob::SwapComplete => f.write_str("Swap complete"),
            Bob::EncSigSent { .. } => f.write_str("Encrypted signature sent"),
        }
    }
}
