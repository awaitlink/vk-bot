use vk_bot::{Bot, Core, Handler};

fn main() {
    let core = Core::new()
        .with_cmd_prefix("/")
        .cmd("info", Handler::new(|ctx| unimplemented!()));

    Bot::new("", "1", 1, "1", 12345, core).start();
}
