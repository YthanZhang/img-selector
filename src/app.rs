use std::path::{Path, PathBuf};

use color_eyre::eyre::WrapErr;
use dark_light::Mode;

use iced::event::Status;
use iced::widget::horizontal_space;
use iced::widget::image::Handle;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, row, text, text_input, vertical_space, Container, Image,
    },
    Application, Command, Event, Length, Subscription, Theme,
};

use crate::move_file::move_file;

const PLACE_HOLDER_IMG: &[u8] =
    include_bytes!("../resource/no_img_placeholder.png");

pub struct App {
    src_dir: Option<String>,
    dest_dirs: [Option<String>; 2],
    img_src_paths: Vec<PathBuf>,
    img_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    SetSourceDir(String),
    SetTargetDir0(String),
    SetTargetDir1(String),
    ImgNext,
    ImgPrev,
    ImgMoveTo(usize),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flag: Self::Flags) -> (App, Command<Message>) {
        let source_dir = std::env::current_dir()
            .wrap_err("Failed to get current directory")
            .and_then(list_image_from_dir)
            .unwrap_or_default();

        (
            Self {
                src_dir: source_dir.first().and_then(|path| {
                    path.parent().map(|p| p.to_string_lossy().to_string())
                }),
                dest_dirs: [None, None],
                img_src_paths: source_dir,
                img_idx: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Image Selector".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::SetSourceDir(dir) => {
                self.src_dir = Some(dir);

                let paths = list_image_from_dir(self.src_dir.as_ref().unwrap())
                    .unwrap_or_default();
                self.img_idx = 0;
                self.img_src_paths = paths;
            }
            Message::SetTargetDir0(dir) => self.dest_dirs[0] = Some(dir),
            Message::SetTargetDir1(dir) => self.dest_dirs[1] = Some(dir),
            Message::ImgNext => {
                if let Some(path) = self.img_src_paths.get(self.img_idx) {
                    if path.exists() {
                        self.img_idx += 1;
                        if self.img_idx >= self.img_src_paths.len() {
                            self.img_idx = 0;
                        }
                    } else {
                        self.update_img_list()
                            .expect("Failed to update image list")
                    }
                }
            }
            Message::ImgPrev => {
                if let Some(path) = self.img_src_paths.get(self.img_idx) {
                    if path.exists() {
                        let (new_value, underflow) =
                            self.img_idx.overflowing_sub(1);
                        if underflow {
                            self.img_idx =
                                self.img_src_paths.len().saturating_sub(1);
                        } else {
                            self.img_idx = new_value;
                        }
                    } else {
                        self.update_img_list()
                            .expect("Failed to update image list")
                    }
                }
            }
            Message::ImgMoveTo(target_idx) => {
                if let Some(source) = self.img_src_paths.get(self.img_idx) {
                    if source.exists() {
                        let target =
                            self.dest_dirs[target_idx].as_ref().map(|dir| {
                                Path::new(dir).join(source.file_name().unwrap())
                            });

                        if let Some(target) = target {
                            move_file(source, target).unwrap();

                            self.img_src_paths.remove(self.img_idx);
                            if self.img_idx >= self.img_src_paths.len() {
                                self.img_idx = 0;
                            }
                            if self.img_src_paths.is_empty() {
                                self.update_img_list()
                                    .expect("Failed to update image list");
                            }
                        }
                    } else {
                        self.update_img_list()
                            .expect("Failed to update image list");
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let top_bar = row([
            text_input(
                "select source directory",
                self.src_dir.as_deref().unwrap_or(""),
            )
            .on_input(Message::SetSourceDir)
            .width(Length::FillPortion(1))
            .into(),
            text_input(
                "select target directory ↑",
                self.dest_dirs[0].as_deref().unwrap_or(""),
            )
            .on_input(Message::SetTargetDir0)
            .width(Length::FillPortion(1))
            .into(),
            text_input(
                "select target directory ↓",
                self.dest_dirs[1].as_deref().unwrap_or(""),
            )
            .on_input(Message::SetTargetDir1)
            .width(Length::FillPortion(1))
            .into(),
        ]
        .into());

        let buttons = column(
            [
                row([
                    horizontal_space(Length::Fill).into(),
                    button(
                        text("Refresh")
                            .horizontal_alignment(Horizontal::Center)
                            .vertical_alignment(Vertical::Center),
                    )
                    .on_press_maybe(
                        self.src_dir
                            .as_ref()
                            .map(|dir| Message::SetSourceDir(dir.clone())),
                    )
                    .height(Length::Shrink)
                    .width(Length::Shrink)
                    .into(),
                    horizontal_space(Length::Fill).into(),
                ]
                .into())
                .height(Length::Fixed(64.0))
                .into(),
                vertical_space(Length::FillPortion(10)).into(),
                button(
                    text("Send to ↑")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press_maybe(
                    self.dest_dirs[0].as_ref().map(|_| Message::ImgMoveTo(0)),
                )
                .height(Length::FillPortion(20))
                .into(),
                vertical_space(Length::FillPortion(10)).into(),
                button(
                    text("Send to ↓")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press_maybe(
                    self.dest_dirs[1].as_ref().map(|_| Message::ImgMoveTo(1)),
                )
                .height(Length::FillPortion(20))
                .into(),
                vertical_space(Length::FillPortion(10)).into(),
                button(
                    text("Next")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::ImgNext)
                .height(Length::FillPortion(20))
                .into(),
                vertical_space(Length::FillPortion(10)).into(),
                button(
                    text("Prev")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::ImgPrev)
                .height(Length::FillPortion(20))
                .into(),
                vertical_space(Length::FillPortion(10)).into(),
            ]
            .into(),
        )
        .height(Length::Fill);

        let image = Container::new(
            Image::new(
                self.img_src_paths
                    .get(self.img_idx)
                    .as_ref()
                    .map(Handle::from_path)
                    .unwrap_or(Handle::from_memory(PLACE_HOLDER_IMG)),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(95))
        .center_x()
        .center_y();



        row([
            buttons.width(Length::FillPortion(10)).into(),
            column([top_bar.into(), image.into()].into())
                .width(Length::FillPortion(90))
                .into(),
        ]
        .into())
        .into()
    }

    fn theme(&self) -> Theme {
        match dark_light::detect() {
            Mode::Dark => Theme::Dark,
            Mode::Light | Mode::Default => Theme::Light,
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::subscription::events_with(keyboard_message)
    }
}


impl App {
    fn update_img_list(&mut self) -> color_eyre::Result<()> {
        self.img_src_paths = self
            .src_dir
            .as_ref()
            .map(list_image_from_dir)
            .unwrap_or(Ok(Vec::new()))?;
        self.img_idx = 0;

        Ok(())
    }
}


fn list_image_from_dir(
    dir: impl AsRef<Path>,
) -> color_eyre::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();

    dir.read_dir()
        .wrap_err("Read directory failed")?
        .map(|path| path.map(|p| p.path()))
        .filter_map(|path| {
            if path
                .as_ref()
                .map(|p| {
                    p.extension()
                        .map(|extension| extension == "png")
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            {
                Some(path)
            } else {
                None
            }
        })
        .collect::<std::io::Result<Vec<_>>>()
        .wrap_err("Read directory path failed")
}


fn keyboard_message(event: Event, status: Status) -> Option<Message> {
    if status == Status::Captured {
        return None;
    }

    match event {
        Event::Mouse(_) | Event::Window(_) | Event::Touch(_) => None,
        Event::Keyboard(key) => {
            if let iced::keyboard::Event::KeyPressed { key_code, modifiers } =
                key
            {
                if modifiers.is_empty() {
                    use iced::keyboard::KeyCode::*;
                    match key_code {
                        Up | W => Some(Message::ImgMoveTo(0)),
                        Down | S => Some(Message::ImgMoveTo(1)),
                        Left | A => Some(Message::ImgPrev),
                        Right | D => Some(Message::ImgNext),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
