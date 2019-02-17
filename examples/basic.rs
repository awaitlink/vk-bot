use vk_bot::{Bot, Core, Event, Handler, Tester};

fn main() {
    // A simple closure for convenience...
    let simple_message_handler = |message| {
        // ...that returns a handler!
        Handler::new(move |ctx| {
            ctx.response().set_message(message);
            eprintln!("{:#?}", ctx.send());
        })
    };

    let core = Core::new()
        .cmd_prefix("/")
        .cmd("info", simple_message_handler("I am a VK bot"))
        .regex("nice", simple_message_handler("Thanks!"))
        .on(
            Event::NoMatch,
            simple_message_handler("Idk how to respond!"),
        )
        .on(
            Event::Start,
            Handler::new(|ctx| {
                ctx.response().set_message("Started!");
                // ctx.response().set_keyboard(...);

                eprintln!("{:#?}", ctx.send());
            }),
        )
        .payload(
            r#"{"a":"b"}"#,
            simple_message_handler("Received {\"a\": \"b\"} in payload!"),
        )
        .dyn_payload(
            Tester::new(|_| true), // accept all remaining payloads!
            simple_message_handler("Received a payload!"),
        );

    Bot::new(
        "",    // VK token
        "1",   // Confirmation token
        1,     // Group ID
        "1",   // Secret (from Callback API settings)
        12345, // Port
        core,
    )
    .start();
}
