use iced::{widget::image, Container, Element, Length, Sandbox, Settings, Svg};

pub fn main() -> iced::Result {
    Tiger::run(Settings::default())
}

struct Tiger;

impl Sandbox for Tiger {
    type Message = ();

    fn new() -> Self {
        Tiger
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, _message: ()) {}

    fn view(&mut self) -> Element<()> {
        let svg = Svg::from_path(String::from(r"C:\Users\Andy\OneDrive\Dokumenter\Andy hjemme\Programmering\Projekte\conway\src\tiger.svg"))
        .width(Length::Fill)
        .height(Length::Fill);

        let mut pixels = vec![];

        for i in 0..256 {
            if i % 2 == 0 {
                for _k in 0..16 {
                    let mut new_pixels = vec![255, 255, 255, 255];
                    pixels.append(&mut new_pixels);
                }
            } else {
                for _k in 0..16 {
                    let mut new_pixels = vec![0, 0, 0, 255];
                    pixels.append(&mut new_pixels);
                }
            }
        }

        println!("{}", pixels.len());
        // println!("{:?}", pixels);

        // let pixels = vec![255, 255, 255, 255, 100, 100, 100, 255, 255, 255, 255, 100, 255, 100, 100, 100];

        // pixels.iter().re

        // let pixels = vec![vec![255, 255, 255, 255, 0, 0, 0, 255], 128];
        // let pixels = pixels.into_iter().flatten().collect();

        let img = image::Handle::from_pixels(64, 64, pixels);
        let img = image::Image::new(img);

        Container::new(img)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
