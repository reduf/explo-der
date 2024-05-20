mod app;
mod clipboard;
mod der;
mod error;
mod hex;
mod oid;
mod oid_names;
mod window;

fn main() {
    let system = window::init("Explo-DER");
    let app = app::App::new();
    system.main_loop(app);
}
