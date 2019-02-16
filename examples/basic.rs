use vk_bot::{Bot, Core, Event, Handler, Tester};

fn main() {
    let core = Core::new()
        .cmd_prefix("/")
        .cmd(
            "info",
            Handler::new(|ctx| {
                ctx.response().set_message("I am a VK bot.");

                eprintln!("{:#?}", ctx.send());
            }),
        )
        .regex(
            "nice",
            Handler::new(|ctx| {
                ctx.response().set_message("Thanks!");

                eprintln!("{:#?}", ctx.send());
            }),
        )
        .on(
            Event::NoMatch,
            Handler::new(|ctx| {
                ctx.response().set_message("Idk how to respond!");

                eprintln!("{:#?}", ctx.send());
            }),
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
            Handler::new(|ctx| {
                ctx.response()
                    .set_message("Received {\"a\": \"b\"} in payload!");

                eprintln!("{:#?}", ctx.send());
            }),
        )
        .dyn_payload(
            Tester::new(|_| true), // accept all remaining payloads!
            Handler::new(|ctx| {
                ctx.response().set_message("Received a payload!");

                eprintln!("{:#?}", ctx.send());
            }),
        );

    Bot::new("", "1", 1, "1", 12345, core).start();
}
