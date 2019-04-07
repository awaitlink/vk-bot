use vk_bot::{Bot, Core, Event, Handler};

fn main() {
    // A simple closure for convenience...
    let simple_handler = |message| {
        // ...that returns a handler!
        Handler::new(move |ctx| {
            // Set the message...
            ctx.response().set_message(message);

            // ...and send it, printing the error if not successful.
            eprintln!("{:?}", ctx.send());
        })
    };

    let core = Core::new()
        // Set prefix for commands (defined via `cmd`):
        .cmd_prefix("/")
        // Command that will be used if message contains `/say_hi` (without quotes) in the beginning:
        .cmd("say_hi", simple_handler("Hi!"))
        // Will be used if message contains `nice` in it:
        .regex("nice", simple_handler("Thanks!"))
        // Will be used if the bot doesn't know how to respond:
        .on(
            Event::NoMatch,
            simple_handler("I don't understand this message..."),
        );

    Bot::new(
        "your vk token",                   // VK token
        "f123456",                         // Confirmation token (from Callback API settings)
        1,                                 // Group ID
        Some("very_secure_phrase".into()), // Secret (from Callback API settings)
        12345,                             // Port
        core,
    )
    .start();
}
