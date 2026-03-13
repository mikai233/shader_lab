mod app;
mod hot_reload;
mod input;
mod renderer;
mod scene;

fn main() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    app::run()
}
