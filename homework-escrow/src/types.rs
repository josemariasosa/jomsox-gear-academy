use gstd::ActorId;
use gstd::prelude::*; // String

pub type Balance = u128;

#[derive(PartialEq, Debug, Encode, Decode, TypeInfo, Clone)]
pub enum EscrowState {
    AwaitingPayment,
    AwaitingDelivery,
    Complete,
}

impl Default for EscrowState {
    fn default() -> Self {
        Self::AwaitingPayment
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum EscrowAction {
    Deposit,
    ConfirmDelivery,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum EscrowEvent {
    Deposited,
    DeliveryConfirmed,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum EscrowMetaState {
    CurrentState,
    Details,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum EscrowMetaStateReply {
    CurrentState(EscrowState),
    Details {
        state: EscrowState,
        buyer: ActorId,
        seller: ActorId,
        price: Balance,
    },
}
