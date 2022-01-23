#![no_std]

use embedded_graphics_core::{Pixel, draw_target::DrawTarget, geometry::Size, geometry::{OriginDimensions}, pixelcolor::*, prelude::{Point}};
use display_interface::DisplayError;

use smart_leds::{SmartLedsWrite, hsv::RGB8};

struct Content<const W: usize, const H: usize>(pub [[RGB8; W]; H]);

impl <const W: usize, const H: usize> Content<W, H> {
    /// Return a slice that aliases the same memory.
    pub fn as_slice(&self) -> &[RGB8] {
        // NOTE(unsafe): Creates a shared reference to the same underlying data,
        // NOTE(unsafe): which we know is tightly packed and so we can compute how many RGB8 pixel is in there.
        unsafe { core::slice::from_raw_parts(self as *const _ as *const RGB8,
                                             core::mem::size_of::<Self>() / core::mem::size_of::<RGB8>()) }
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
    fn new(writer: T, matrix_type: M) -> Self {
        let content = Content::<W, H>([[RGB8::default(); W]; H]);
        Self{writer: writer,
            content: content,
            matrix_type: matrix_type}
    }
}

impl<T: SmartLedsWrite, M: MatrixType, const W: usize, const H: usize> DrawTarget for SmartLedMatrix<T, M, W, H> 
where <T as SmartLedsWrite>::Color: From<RGB8> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
    I: IntoIterator<Item = Pixel<Rgb888>> {
        let mut out_of_bounds_checker: Result<(), DisplayError> = Ok(());
        pixels.into_iter().for_each(|Pixel(pos, color)| {
            match self.matrix_type.map(pos) {
                Ok(mapped_pos) => self.content.0[mapped_pos.x as usize][mapped_pos.y as usize] = RGB8::new(color.r(), color.g(), color.b()),
                Err(e) => out_of_bounds_checker = Err(e),
            }
        });
        let iter = self.content.as_slice().iter().cloned();
        match self.writer.write(iter) {
            Ok(()) => {
                out_of_bounds_checker
            }
            Err(_) => {
                Err(DisplayError::BusWriteError)
            }
        }
    }
}

pub trait MatrixType {
    fn map(&self, pos: Point) -> Result<Point, DisplayError>;
    fn size(&self) -> Size;
}

pub struct MT8x8 {
}

pub fn new_8x8<T: SmartLedsWrite>(writer: T) -> SmartLedMatrix<T, MT8x8, 8, 8> {
    SmartLedMatrix::<_, _, 8, 8>::new(writer, MT8x8{})
}

impl MatrixType for MT8x8 {
    fn map(&self, pos: Point) -> Result<Point, DisplayError> {
        if pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7 {
            Ok(Point::new(pos.x, 7 - pos.y))
        } else {
            Err(DisplayError::OutOfBoundsError)
        }
    }

    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}
