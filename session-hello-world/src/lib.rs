#![no_std]
use gstd::{msg, debug, ActorId};
use gstd::prelude::*; // String

static mut GREETING: String = String::new();
static mut ADMIN:  ActorId = ActorId::zero();

#[derive(Debug, Encode, Decode, TypeInfo)]
enum Message {
    SetNewGreeting { greeting: String },
    Greet,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let message: Message = msg::load().expect("Could not load Message");
    match message {
        Message::SetNewGreeting { greeting } => {
            assert_eq!(msg::source(), ADMIN, "Only admin can set new greeting");
            GREETING = greeting;
            debug!(
                "Greeting was set to '{}'",
                GREETING
            );
        },
        Message::Greet => {
            msg::reply_bytes(GREETING.clone(), 0).unwrap();
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    GREETING = String::from_utf8(msg::load_bytes()).expect("Invalid message");
    ADMIN = msg::source();
    debug!(
      "Program was initialized with '{}' greeting and '{:?}' admin", 
      GREETING, ADMIN
    );
}

#[cfg(test)]
 mod tests {
    use gstd::ToString;
    use gtest::{Program, System};
    use super::Message;

    fn init(system: &System) {
        system.init_logger();

        let program = Program::current(&system);

        let greeting = "Hi";
        let res = program.send_bytes(2, greeting);
        assert!(res.log().is_empty());
    }

    #[test]
    fn set_new_greeting() {
        let system = System::new();
        init(&system);

        let program = system.get_program(1);
        let res = program.send(
            2,
            Message::SetNewGreeting {
                greeting: "Hello".to_string(),
            },
        );
        assert!(res.log().is_empty());

        let res = program.send(3, Message::Greet);
        assert!(res.contains(&(3, "Hello")));
    }

    #[test]
    fn fail() {
        let system = System::new();
        init(&system);

        let program = system.get_program(1);
        let res = program.send(
            3,
            Message::SetNewGreeting {
                greeting: "Hello".to_string(),
            },
        );
        assert!(res.main_failed());
    }
}