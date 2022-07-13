#![no_std]
use gstd::{msg, debug};
use gstd::prelude::*; // String

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let message = String::from_utf8(msg::load_bytes()).expect("Invalid message");
    debug!("Message: {}", message);
    assert!(message.contains("PING"), "PING not in message!");

    let response = "PONG".to_string();
    msg::reply_bytes(response, 0).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn init() {}

#[cfg(test)]
 mod tests {
    use super::String;
    use gtest::{Program, System};

    fn init(system: &System) {
        system.init_logger();

        let program = Program::current(&system);

        let greeting = "Hi";
        let res = program.send_bytes(2, greeting);
        assert!(res.log().is_empty());
    }

    #[test]
    fn test_ping() {
        let system = System::new();
        init(&system);

        let program = system.get_program(1);
        let res = program.send_bytes(2, "This is a PING message");
        assert!(res.contains(&(2, "PONG")));
    }

    #[test]
    fn fail() {
        let system = System::new();
        init(&system);

        let program = system.get_program(1);
        let res = program.send(
            2,
            String::from("ping")
        );
        assert!(res.main_failed());
    }
}