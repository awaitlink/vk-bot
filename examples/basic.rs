use vk_bot::{Bot, Core, Handler};

fn main() {
    let core = Core::new().with_cmd_prefix("/").cmd(
        "info",
        Handler::new(|ctx| {
            ctx.response().set_message("I am a VK bot!");
            let _ = ctx.send();
        }),
    );

    Bot::new("", "1", 1, "1", 12345, core).start();
}
