mod app;
mod model;
mod render;
mod style;

fn main() -> Result<(), eframe::Error> {
    app::run()
}
