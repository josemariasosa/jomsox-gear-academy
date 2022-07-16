#![no_std]
use gstd::{msg, exec, ActorId};
use gstd::prelude::*; // String

mod types;
use crate::types::*;

#[derive(Encode, Decode)]
pub struct InitLoan {
    amount: Balance,
    interest: Balance,
    lender: ActorId,
    borrower: ActorId,
    duration: u64,
}

#[derive(Default)]
pub struct Loan {
    state: LoanState,
    borrower: ActorId,
    lender: ActorId,
    duration: u64,
    end: u64,
    amount: Balance,
    interest: Balance,
}

impl Loan {
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

    fn check_state(&self, state: LoanState) {
        assert_eq!(
            self.state, state,
            "Loan must be in the {:?} state", state
        );
    }

    fn fund(&mut self) {
        self.check_msg_source(self.lender);
        self.check_attached_value(self.amount);
        self.check_state(LoanState::Pending);

        self.state = LoanState::Active;        
        self.end = exec::block_timestamp() + self.duration;

        msg::send(self.borrower, b"Lending is active", msg::value()).expect("Error in sending value to lender");
        msg::reply(LoanEvent::Funded, 0).expect("Error in reply [LoanEvent::Funded]");
    }

    fn reimburse(&mut self) {
        self.check_msg_source(self.borrower);
        self.check_attached_value(self.amount + self.interest);
        self.check_state(LoanState::Active);
        assert!(exec::block_timestamp() >= self.end, "Too early for reimbursement");

        self.state = LoanState::Closed;

        msg::send(self.lender, b"Reimburse", msg::value()).expect("Error in sending value to lender");
        msg::reply(LoanEvent::Reimbursed, 0).expect("Error in reply [LoanEvent::Reimbursed]");
    }
}

impl Default for LoanState {
    fn default() -> Self {
        Self::Pending
    }
}

static mut LOAN: Option<Loan> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let loan_config: InitLoan = msg::load().expect("Unable to decode InitLoan");
    let loan = Loan {
        borrower: loan_config.borrower,
        lender: loan_config.lender,
        duration: loan_config.duration,
        amount: loan_config.amount,
        interest: loan_config.interest,
        ..Loan::default()
    };
    LOAN = Some(loan);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: LoanAction = msg::load().expect("Could not load LoanAction");
    let loan: &mut Loan = LOAN.get_or_insert(Loan::default());
    match action {
        LoanAction::Fund => loan.fund(),
        LoanAction::Reimburse => loan.reimburse(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: LoanMetaState = msg::load().expect("failed to decode LoanMetaState");
    let loan: &mut Loan = LOAN.get_or_insert(Loan::default());
    let encoded = match state {
        LoanMetaState::CurrentState => {
            LoanMetaStateReply::CurrentState(loan.state.clone()).encode()
        }
        LoanMetaState::Details => LoanMetaStateReply::Details {
            lender: loan.lender,
            borrower: loan.borrower,
            amount: loan.amount,
            interest: loan.interest,
            end: loan.end,
        }
        .encode(),
    };
    gstd::util::to_leak_ptr(encoded)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use gtest::{Program, RunResult, System};

    pub const AMOUNT: Balance = 1000;
    pub const INTEREST: Balance = 100;
    pub const LENDER: u64 = 2;
    pub const BORROWER: u64 = 3;
    pub const DURATION: u64 = 10 * 24 * 60 * 60 * 1000;

    fn init_loan(sys: &System) {
        sys.init_logger();
        let loan = Program::current(&sys);
        let res = loan.send(
            LENDER,
            InitLoan {
                amount: AMOUNT,
                interest: INTEREST,
                lender: LENDER.into(),
                borrower: BORROWER.into(),
                duration: DURATION,
            },
        );
        assert!(res.log().is_empty());
    }

    fn fund(loan: &Program, from: u64, amount: Balance) -> RunResult {
        loan.send_with_value(from, LoanAction::Fund, amount)
    }
    fn reimburse(loan: &Program, from: u64, amount: Balance) -> RunResult {
        loan.send_with_value(from, LoanAction::Reimburse, amount)
    }

    #[test]
    fn fund_success() {
        let sys = System::new();
        init_loan(&sys);
        let loan = sys.get_program(1);
        let res = fund(&loan, LENDER, AMOUNT);
        assert!(res.contains(&(LENDER, LoanEvent::Funded.encode())));
    }

    #[test]
    fn fund_failures() {
        let sys = System::new();
        init_loan(&sys);
        let loan = sys.get_program(1);
        // must fail since the caller account is not a lender
        assert!(fund(&loan, BORROWER, AMOUNT).main_failed());
        // must fail since attached value is not equal to the amount indicated in the contract
        assert!(fund(&loan, LENDER, 1001).main_failed());

        // funded
        assert!(!fund(&loan, LENDER, AMOUNT).main_failed());
        sys.spend_blocks(DURATION as u32);
        // reimbursed
        assert!(!reimburse(&loan, BORROWER, 1100).main_failed());

        // must fail since loan is already closed
        assert!(fund(&loan, LENDER, AMOUNT).main_failed());
    }

    #[test]
    fn reimburse_success() {
        let sys = System::new();
        init_loan(&sys);
        let loan = sys.get_program(1);

        let res = fund(&loan, LENDER, AMOUNT);
        assert!(res.contains(&(LENDER, LoanEvent::Funded.encode())));

        sys.spend_blocks(DURATION as u32);

        let res = reimburse(&loan, BORROWER, AMOUNT + INTEREST);
        assert!(res.contains(&(BORROWER, LoanEvent::Reimbursed.encode())));
    }

    #[test]
    fn reimburse_failures() {
        let sys = System::new();
        init_loan(&sys);
        let loan = sys.get_program(1);
        // must fail since the caller account is not a lender
        assert!(fund(&loan, BORROWER, 1000).main_failed());
        // must fail since attached value is not equal to the amount indicated in the contract
        assert!(fund(&loan, LENDER, 1001).main_failed());

        // funded
        assert!(!fund(&loan, LENDER, 1000).main_failed());
        sys.spend_blocks(DURATION as u32);
        // reimbursed
        assert!(!reimburse(&loan, BORROWER, 1100).main_failed());

        // must fail since loan is already closed
        assert!(fund(&loan, LENDER, 1000).main_failed());
    }
}