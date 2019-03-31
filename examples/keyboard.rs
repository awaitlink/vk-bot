use vk_bot::{
    keyboard::{Button, Color, Keyboard},
    Bot, Core, Handler, Tester,
};

fn main() {
    // Simple handler, see `examples/basic.rs` for more details.
    let simple_handler = |message| {
        Handler::new(move |ctx| {
            ctx.response().set_message(message);
            eprintln!("{:?}", ctx.send());
        })
    };

    // Create a keyboard.
    let kbd = Keyboard::new(
        // Vec of rows
        vec![
            // Row 0
            vec![
                Button::new("A", Color::Primary, None),
                Button::new("B", Color::Default, Some(r#"{"a": "b"}"#.into())),
            ],
        ],
        false, // One-time? (i.e. show only until a button is pressed on the keyboard?)
    );

    let core = Core::new()
        // Set prefix for commands (defined via `cmd`):
        .cmd_prefix("/")
        // Command that will be used if message contains `/keyboard` (without quotes) in the beginning:
        .cmd(
            "keyboard",
            Handler::new(move |ctx| {
                ctx.response().set_message("Here you go:");
                ctx.response().set_keyboard(kbd.clone());

                eprintln!("{:?}", ctx.send());
            }),
        )
        // Used when the specified payload is found inside of the message:
        .payload(r#"{"a":"b"}"#, simple_handler("You pressed button B!"))
        // Accept all remaining payloads:
        .dyn_payload(Tester::new(|_| true), simple_handler("Received a payload!"));

    Bot::new(
        "your vk token",      // VK token
        "f123456",            // Confirmation token (from Callback API settings)
        1,                    // Group ID
        "very_secure_phrase", // Secret (from Callback API settings)
        12345,                // Port
        core,
    )
    .start();
}
