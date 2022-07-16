#![no_std]
use gstd::{msg, ActorId};
use gstd::prelude::*; // String

mod types;
use crate::types::*;

#[derive(Encode, Decode)]
pub struct InitEscrow {
    price: Balance,
    buyer: ActorId,
    seller: ActorId,
}

#[derive(Default)]
pub struct Escrow {
    state: EscrowState,
    buyer: ActorId,
    seller: ActorId,
    price: Balance,
}

impl Escrow {
    fn check_msg_source(&self, account: ActorId) {
        assert_eq!(
            msg::source(), account,
            "`msg::source` must be {:?} account", account
        );
    }

    fn check_attached_value(&self, value: Balance) {
        assert_eq!(
            msg::value(), value,
            "Must attached the {:?} value", value
        );
    }

    fn check_state(&self, state: EscrowState) {
        assert_eq!(
            self.state, state,
            "Loan must be in the {:?} state", state
        );
    }

    fn deposit(&mut self) {
        self.check_msg_source(self.buyer);
        self.check_attached_value(self.price);
        self.check_state(EscrowState::AwaitingPayment);

        self.state = EscrowState::AwaitingDelivery;        

        msg::reply(EscrowEvent::Deposited, 0).expect("Error in reply [EscrowEvent::Deposited]");
    }

    fn confirm_delivery(&mut self) {
        self.check_msg_source(self.buyer);
        self.check_state(EscrowState::AwaitingDelivery);

        self.state = EscrowState::Complete;

        msg::send(self.seller, b"Delivery Confirmed", self.price).expect("Error in sending value to seller");
        msg::reply(EscrowEvent::DeliveryConfirmed, 0).expect("Error in reply [EscrowEvent::DeliveryConfirmed]");
    }
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let escrow_config: InitEscrow = msg::load().expect("Unable to decode InitEscrow");
    let escrow = Escrow {
        buyer: escrow_config.buyer,
        seller: escrow_config.seller,
        price: escrow_config.price,
        ..Escrow::default()
    };
    ESCROW = Some(escrow);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: EscrowAction = msg::load().expect("Could not load EscrowAction");
    let contract: &mut Escrow = ESCROW.get_or_insert(Escrow::default());
    match action {
        EscrowAction::Deposit => contract.deposit(),
        EscrowAction::ConfirmDelivery => contract.confirm_delivery(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: EscrowMetaState = msg::load().expect("failed to decode EscrowMetaState");
    let contract: &mut Escrow = ESCROW.get_or_insert(Escrow::default());
    let encoded = match state {
        EscrowMetaState::CurrentState => {
            EscrowMetaStateReply::CurrentState(contract.state.clone()).encode()
        }
        EscrowMetaState::Details => EscrowMetaStateReply::Details {
            state: contract.state.clone(),
            buyer: contract.buyer,
            seller: contract.seller,
            price: contract.price,
        }
        .encode(),
    };
    gstd::util::to_leak_ptr(encoded)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use gtest::{Program, RunResult, System};
    use gstd::debug;

    pub const PRICE: Balance = 1000;
    pub const SELLER: u64 = 5;
    pub const BUYER: u64 = 6;

    fn init_contract(sys: &System) {
        sys.init_logger();
        let contract = Program::current(&sys);
        let res = contract.send(
            SELLER,
            InitEscrow {
                price: PRICE,
                seller: SELLER.into(),
                buyer: BUYER.into(),
            },
        );
        assert!(res.log().is_empty());
    }

    fn deposit(contract: &Program, from: u64, amount: Balance) -> RunResult {
        contract.send_with_value(from, EscrowAction::Deposit, amount)
    }

    fn confirm_delivery(contract: &Program, from: u64) -> RunResult {
        contract.send(from, EscrowAction::ConfirmDelivery)
    }

    #[test]
    fn deposit_success() {
        let sys = System::new();
        init_contract(&sys);
        let contract = sys.get_program(1);
        // debug!("Balance: {:?}", 1);//sys.balance_of(SELLER));
        sys.mint_to(BUYER, PRICE);
        let res = deposit(&contract, BUYER, PRICE);
        assert!(res.contains(&(BUYER, EscrowEvent::Deposited.encode())));
    }

    // #[test]
    // fn fund_failures() {
    //     let sys = System::new();
    //     init_contract(&sys);
    //     let loan = sys.get_program(1);
    //     // must fail since the caller account is not a lender
    //     assert!(fund(&loan, BORROWER, AMOUNT).main_failed());
    //     // must fail since attached value is not equal to the amount indicated in the contract
    //     assert!(fund(&loan, LENDER, 1001).main_failed());

    //     // funded
    //     assert!(!fund(&loan, LENDER, AMOUNT).main_failed());
    //     sys.spend_blocks(DURATION as u32);
    //     // reimbursed
    //     assert!(!reimburse(&loan, BORROWER, 1100).main_failed());

    //     // must fail since loan is already closed
    //     assert!(fund(&loan, LENDER, AMOUNT).main_failed());
    // }

    // #[test]
    // fn reimburse_success() {
    //     let sys = System::new();
    //     init_contract(&sys);
    //     let loan = sys.get_program(1);

    //     let res = fund(&loan, LENDER, AMOUNT);
    //     assert!(res.contains(&(LENDER, LoanEvent::Funded.encode())));

    //     sys.spend_blocks(DURATION as u32);

    //     let res = reimburse(&loan, BORROWER, AMOUNT + INTEREST);
    //     assert!(res.contains(&(BORROWER, LoanEvent::Reimbursed.encode())));
    // }

    // #[test]
    // fn reimburse_failures() {
    //     let sys = System::new();
    //     init_contract(&sys);
    //     let loan = sys.get_program(1);
    //     // must fail since the caller account is not a lender
    //     assert!(fund(&loan, BORROWER, 1000).main_failed());
    //     // must fail since attached value is not equal to the amount indicated in the contract
    //     assert!(fund(&loan, LENDER, 1001).main_failed());

    //     // funded
    //     assert!(!fund(&loan, LENDER, 1000).main_failed());
    //     sys.spend_blocks(DURATION as u32);
    //     // reimbursed
    //     assert!(!reimburse(&loan, BORROWER, 1100).main_failed());

    //     // must fail since loan is already closed
    //     assert!(reimburse(&loan, BORROWER, 1100).main_failed());
    // }
}