#![no_std]
#![feature(min_const_generics)]

use embedded_graphics_core::{Pixel, draw_target::DrawTarget, geometry::Size, geometry::{Dimensions, OriginDimensions}, pixelcolor::*, prelude::{Point, PointsIter}, primitives::Rectangle};
use display_interface::DisplayError;

use smart_leds::{SmartLedsWrite, hsv::RGB8};

struct Content<const W: usize, const H: usize>(pub [[RGB8; W]; H]);

impl <const W: usize, const H: usize> Content<W, H> {
    /// Return a slice that aliases the same memory.
    pub fn as_slice(&self) -> &[RGB8] {
        // NOTE(unsafe): Creates a shared reference to the same underlying data,
        // NOTE(unsafe): which we know is tightly packed and so a valid [u8].
        unsafe { core::slice::from_raw_parts(self as *const _ as *const RGB8,
                                             core::mem::size_of::<Self>()) }
    }
}

pub struct SmartLedMatrix<T, M: MatrixType, const W: usize, const H: usize> {
    writer: T,
    content: Content<W, H>,
    matrix_type: M
}

impl<T: SmartLedsWrite, M: MatrixType, const W: usize, const H: usize> OriginDimensions for SmartLedMatrix<T, M, W, H> {
    fn size(&self) -> Size {
        self.matrix_type.size()
    }
}

impl<T: SmartLedsWrite, M: MatrixType, const W: usize, const H: usize> SmartLedMatrix<T, M, W, H> {
    pub fn new(writer: T, matrix_type: M) -> Self {
        let content = Content::<W, H>([[RGB8::default(); W]; H]);
        Self{writer: writer,
            content: content,
            matrix_type: matrix_type}
    }
}

impl<T: SmartLedsWrite, M: MatrixType, const w: usize, const h: usize> DrawTarget for SmartLedMatrix<T, M, w, h> 
where <T as SmartLedsWrite>::Color: From<RGB8> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
    I: IntoIterator<Item = Pixel<Rgb888>> {
        pixels.into_iter().for_each(|Pixel(pos, color)| {
            if self.matrix_type.position_valid(pos) {
                self.content.0[pos.x as usize][pos.y as usize] = RGB8::new(color.r(), color.g(), color.b());
            }
        });
        //TODO: always returns an SPI overrun error on my stm32f401 
        let iter = self.content.as_slice().iter().cloned();
        match self.writer.write(iter) {
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
