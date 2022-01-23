#![no_std]
#![feature(generic_const_exprs)]

use embedded_graphics_core::{Pixel, draw_target::DrawTarget, geometry::Size, geometry::{Dimensions, OriginDimensions}, pixelcolor::*, prelude::{Point, PointsIter}, primitives::Rectangle};
use display_interface::DisplayError;

use smart_leds::{SmartLedsWrite, hsv::RGB8};

pub struct SmartLedMatrix<T, M: MatrixType> 
where [(); M::PIXELS]: {
    writer: T,
    content: [RGB8; M::PIXELS],
    matrix_type: M
}

impl<T: SmartLedsWrite, M: MatrixType> OriginDimensions for SmartLedMatrix<T, M> 
where [(); M::PIXELS]: {
    fn size(&self) -> Size {
        self.matrix_type.size()
    }
}

impl<T: SmartLedsWrite, M: MatrixType> SmartLedMatrix<T, M> 
where [(); M::PIXELS]: {
    pub fn new(writer: T, matrix_type: M) -> Self {
        Self{writer: writer,
            content: [RGB8::default(); M::PIXELS],
            matrix_type: matrix_type}
    }
}

impl<T: SmartLedsWrite, M: MatrixType> DrawTarget for SmartLedMatrix<T, M> 
where <T as SmartLedsWrite>::Color: From<RGB8>,
[(); M::PIXELS]: {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
    I: IntoIterator<Item = Pixel<Rgb888>> {
        pixels.into_iter().for_each(|Pixel(pos, color)| {
            if self.matrix_type.position_valid(pos) {
                self.content[self.matrix_type.map(pos.x, pos.y)] = RGB8::new(color.r(), color.g(), color.b());
            }
        });
        //TODO: always returns an SPI overrun error on my stm32f401 
        match self.writer.write(self.content.iter().cloned()) {
            Ok(()) => {
                Ok(())
            }
            Err(_) => {
                Err(DisplayError::BusWriteError)
            }
        }
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

pub trait MatrixType {
    fn map(&self, x: i32, y: i32) -> usize;
    fn position_valid(&self, pos: Point) -> bool;
    fn size(&self) -> Size;
    const PIXELS: usize;
}

pub struct MT8x8 {
}

impl MatrixType for MT8x8 {
    fn map(&self, x: i32, y: i32) -> usize {
        (x*8+(7-y)) as usize
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }

    const PIXELS: usize = 64;

    fn position_valid(&self, pos: Point) -> bool {
        pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7
    }
}
