use gstd::ActorId;
use gstd::prelude::*; // String

pub type Balance = u128;

#[derive(PartialEq, Debug, Encode, Decode, TypeInfo, Clone)]
pub enum LoanState {
    Pending,
    Active,
    Closed,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum LoanAction {
    Fund,
    Reimburse,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum LoanEvent {
    Funded,
    Reimbursed,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum LoanMetaState {
    CurrentState,
    Details,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum LoanMetaStateReply {
    CurrentState(LoanState),
    Details {
        lender: ActorId,
        borrower: ActorId,
        amount: u128,
        interest: u128,
        end: u64,
    },
}
