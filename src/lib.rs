#![no_std]

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    pixelcolor::*,
    Pixel,
    primitives::Rectangle,
    prelude::PointsIter,
};
use display_interface::DisplayError;

//use smart_leds_trait::{SmartLedsWrite, RGB8};
use smart_leds::{SmartLedsWrite, hsv::RGB8};
use ws2812_spi as ws2812;
pub struct SmartLedMatrix<T> {
    writer: ws2812::Ws2812<T>,
    content: [RGB8; 64]
}


impl<T: embedded_hal::spi::FullDuplex<u8>> OriginDimensions for SmartLedMatrix<T> {
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}


impl<T: embedded_hal::spi::FullDuplex<u8>> SmartLedMatrix<T> {
    pub fn new(writer: ws2812::Ws2812<T>) -> Self {
            Self{writer: writer,
                content: [RGB8::default(); 64]}
    }
}

impl<T: embedded_hal::spi::FullDuplex<u8>> DrawTarget for SmartLedMatrix<T> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
    I: IntoIterator<Item = Pixel<Rgb888>> {
        pixels.into_iter().for_each(|Pixel(pos, color)| {
            if pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7 {
                self.content[(pos.x*8+pos.y) as usize] = RGB8::new(color.r(), color.g(), color.b());
            }
        });
        //TODO: always returns an SPI overrun error on my stm32f401 
        self.writer.write(self.content.iter().cloned());
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(pos, color)),
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }

}
