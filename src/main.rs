#![feature(io_error_more)]

use iced::Application;

mod app;
mod move_file;

fn main() -> color_eyre::Result<()> {
    app::App::run(iced::Settings::default())?;
    Ok(())
}
