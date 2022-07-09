#![no_std]
use gstd::{msg, String, debug, ActorId};

static mut GREETING: String = String::new();
static mut ADMIN:  ActorId = ActorId::new([0u8; 32]);

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


    // let new_msg = String::from_utf8(msg::load_bytes()).expect("Invalid message");
    // if new_msg == "Hello" {
    //     msg::reply(b"Hello!", 0).expect("Error in sending reply");
    // }

    // msg::reply_bytes(GREETING.clone(),0).expect("Error in sending reply");
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
    use gtest::{Program, System};

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

//     #[test]
//     fn test_hello() {
//         let system = System::new();
//         system.init_logger();
//         let program = Program::current(&system);

//         let greeting = "Hi";
//         let res = program.send_bytes(2, greeting);
//         assert!(res.log().is_empty());

//         // let res = program.send_bytes(2, "Hello");
//         // let reply = "Hello!".as_bytes();
//         // assert!(res.contains(&(2, reply)));
//         let res = program.send_bytes(2, "");
//         assert!(res.contains(&(2, greeting)));
//    }
}