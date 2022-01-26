//! # Smart Leds Matrix
//!
//! This is a library that adapts [smart-leds](https://crates.io/crates/smart-leds) driver implementations to the
//! [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) crate by wrapping the LED
//! driver into a `Drawable` display target.
//!

#![no_std]

use core::marker::PhantomData;

use display_interface::DisplayError;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

use smart_leds::{brightness, hsv::RGB8, SmartLedsWrite};

struct Content<const W: usize, const H: usize>(pub [[RGB8; W]; H]);

impl<const W: usize, const H: usize> Content<W, H> {
    /// Return a slice that aliases the same memory.
    pub fn as_slice(&self) -> &[RGB8] {
        // NOTE(unsafe): Creates a shared reference to the same underlying data,
        // NOTE(unsafe): which we know is tightly packed and so we can compute how many RGB8 pixel is in there.
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const RGB8,
                core::mem::size_of::<Self>() / core::mem::size_of::<RGB8>(),
            )
        }
    }
}

/// The wrapper for the LED driver.
///
/// This receives the `SmartLedsWriter` trait implementations along with a
/// `MatrixType` that describes the size and the pixels mapping between the LED
/// strip placement and the matrix's x y coordinates.
pub struct SmartLedMatrix<T, M: MatrixType<W, H>, const W: usize, const H: usize> {
    writer: T,
    content: Content<W, H>,
    matrix_type: PhantomData<M>,
    brightness: u8,
}

impl<T, M: MatrixType<W, H>, const W: usize, const H: usize> SmartLedMatrix<T, M, W, H> {
    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }
}

impl<T: SmartLedsWrite, M: MatrixType<W, H>, const W: usize, const H: usize> OriginDimensions
    for SmartLedMatrix<T, M, W, H>
{
    fn size(&self) -> Size {
        Size::new(W as u32, H as u32)
    }
}

impl<T: SmartLedsWrite, M: MatrixType<W, H>, const W: usize, const H: usize>
    SmartLedMatrix<T, M, W, H>
where
    <T as SmartLedsWrite>::Color: From<RGB8>,
{
    pub fn new(writer: T) -> Self {
        let content = Content::<W, H>([[RGB8::default(); W]; H]);
        Self {
            writer,
            content,
            matrix_type: PhantomData,
            brightness: 255,
        }
    }
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        let iter = brightness(self.content.as_slice().iter().cloned(), self.brightness);
        match self.writer.write(iter) {
            Ok(()) => Ok(()),
            Err(_) => Err(DisplayError::BusWriteError),
        }
    }
}

impl<T: SmartLedsWrite, M: MatrixType<W, H>, const W: usize, const H: usize> DrawTarget
    for SmartLedMatrix<T, M, W, H>
where
    <T as SmartLedsWrite>::Color: From<RGB8>,
{
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(pos, color) in pixels {
            if let Some(mapped_pos) = M::map(pos) {
                self.content.0[mapped_pos.x as usize][mapped_pos.y as usize] =
                    RGB8::new(color.r(), color.g(), color.b());
            }
        }

        Ok(())
    }
}

/// Trait that represents a certain type of LED matrix.
///
/// The map() function shall fix any x y coordinate mismatch. Mismatch means
/// the matrix might display the result being drawn in mirrored or otherwise
/// incorrect ways due to the LEDs order on the PCB.
/// The size() function returns the x and y size of the matrix to satisfy
/// embedded-graphics user libraries.
pub trait MatrixType<const W: usize, const H: usize> {
    fn map(pos: Point) -> Option<Point>;
}

/// Type definition for simple 8x8 matrix.
pub enum InvertY {}

/// Factory function that wraps the LED driver and produces the appropriate SmartLedsMatrix.
///
/// User should use this function to work with the crate.
pub fn new_8x8<T: SmartLedsWrite>(writer: T) -> SmartLedMatrix<T, InvertY, 8, 8>
where
    <T as SmartLedsWrite>::Color: From<RGB8>,
{
    SmartLedMatrix::new(writer)
}

impl<const W: usize, const H: usize> MatrixType<W, H> for InvertY {
    fn map(pos: Point) -> Option<Point> {
        let width = W as i32;
        let height = H as i32;

        (pos.x >= 0 && pos.x < width && pos.y >= 0 && pos.y < height)
            .then(|| Point::new(pos.x, (height - 1) - pos.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWriter {}

    impl SmartLedsWrite for MockWriter {
        type Error = ();
        type Color = RGB8;

        fn write<T, I>(&mut self, _iterator: T) -> Result<(), Self::Error>
        where
            T: Iterator<Item = I>,
            I: Into<Self::Color>,
        {
            todo!()
        }
    }

    #[test]
    fn test() {
        let writer = MockWriter {};

        let _matrix = SmartLedMatrix::<_, InvertY, 8, 8>::new(writer);
    }
}
