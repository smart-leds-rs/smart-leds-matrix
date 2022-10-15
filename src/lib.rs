//! # Smart Leds Matrix
//!
//! This is a library that adapts [smart-leds](https://crates.io/crates/smart-leds) driver implementations to the
//! [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) crate by wrapping the LED
//! driver into a `Drawable` display target.
//!

#![no_std]

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

use smart_leds::{brightness, hsv::RGB8, SmartLedsWrite};

pub mod layout;
use layout::Layout;

/// The wrapper for the LED driver.
///
/// This receives the `SmartLedsWriter` trait implementations along with a
/// `Transformation` that describes the pixels mapping between the LED
/// strip placement and the matrix's x y coordinates.
pub struct SmartLedMatrix<T, L, const N: usize> {
    writer: T,
    layout: L,
    content: [RGB8; N],
    brightness: u8,
}

impl<T, L, const N: usize> SmartLedMatrix<T, L, N> {
    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }
}

impl<T: SmartLedsWrite, L: Layout, const N: usize> SmartLedMatrix<T, L, N>
where
    <T as SmartLedsWrite>::Color: From<RGB8>,
{
    pub fn new(writer: T, layout: L) -> Self {
        Self {
            writer,
            layout,
            content: [RGB8::default(); N],
            brightness: 255,
        }
    }

    pub fn flush(&mut self) -> Result<(), T::Error> {
        let iter = brightness(self.content.as_slice().iter().cloned(), self.brightness);
        self.writer.write(iter)
    }
}

impl<T: SmartLedsWrite, L: Layout, const N: usize> DrawTarget for SmartLedMatrix<T, L, N>
where
    <T as SmartLedsWrite>::Color: From<RGB8>,
{
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(pos, color) in pixels {
            if let Some(t) = self
                .layout
                .map(pos)
                .and_then(|index| self.content.get_mut(index))
            {
                *t = RGB8::new(color.r(), color.g(), color.b());
            }
        }

        Ok(())
    }
}

impl<T, L: Layout, const N: usize> OriginDimensions for SmartLedMatrix<T, L, N> {
    fn size(&self) -> Size {
        self.layout.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::Rectangular;
    use embedded_graphics_core::{geometry::Point, prelude::Dimensions, primitives::PointsIter};

    struct MockWriter<'a, const N: usize> {
        content: &'a mut [RGB8; N],
    }

    impl<'a, const N: usize> SmartLedsWrite for MockWriter<'a, N> {
        type Error = ();
        type Color = RGB8;

        fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
        where
            T: Iterator<Item = I>,
            I: Into<Self::Color>,
        {
            let mut i = 0;
            for color in iterator {
                self.content[i] = color.into();
                i += 1;
            }
            Ok(())
        }
    }

    fn get64pixels(color: Rgb888) -> ([Pixel<Rgb888>; 64]) {
        let mut pixels: [Pixel<Rgb888>; 64] = [Pixel(Point::new(0, 0), Rgb888::BLACK); 64];
        for x in 0..8 {
            for y in 0..8 {
                pixels[x * 8 + y] = Pixel(Point::new(x as i32, y as i32), color);
            }
        }
        pixels
    }

    #[test]
    fn test_y_inversion() {
        let content = &mut [RGB8::new(0, 0, 0); 64];
        let writer = MockWriter { content };
        let mut matrix =
            SmartLedMatrix::<_, _, { 8 * 8 }>::new(writer, Rectangular::new_invert_y(8, 8));
        let mut pixels = get64pixels(Rgb888::BLACK);

        pixels[0] = Pixel(Point::new(0, 0), Rgb888::WHITE);

        matrix.draw_iter(pixels).unwrap();
        matrix.flush().unwrap();

        for i in 0..64 {
            if i == 56 {
                assert_eq!(
                    content[i],
                    RGB8::new(255, 255, 255),
                    r#"expected a white pixel after inversion"#
                );
                continue;
            }
            assert_eq!(content[i], RGB8::new(0, 0, 0), r#"expected black pixel"#);
        }
    }

    #[test]
    fn test_identity() {
        let content = &mut [RGB8::new(0, 0, 0); 64];
        let writer = MockWriter { content };
        let mut matrix = SmartLedMatrix::<_, _, { 8 * 8 }>::new(writer, Rectangular::new(8, 8));
        let mut pixels = get64pixels(Rgb888::BLACK);

        pixels[0] = Pixel(Point::new(0, 0), Rgb888::WHITE);

        matrix.draw_iter(pixels).unwrap();
        matrix.flush().unwrap();

        for i in 0..64 {
            if i == 0 {
                assert_eq!(
                    content[i],
                    RGB8::new(255, 255, 255),
                    r#"expected a white pixel on it's original place"#
                );
                continue;
            }
            assert_eq!(content[i], RGB8::new(0, 0, 0), r#"expected black pixel"#);
        }
    }

    #[test]
    fn test_brightness() {
        let content = &mut [RGB8::new(0, 0, 0); 64];
        let writer = MockWriter { content };
        let mut matrix = SmartLedMatrix::<_, _, { 8 * 8 }>::new(writer, Rectangular::new(8, 8));
        let pixels = get64pixels(Rgb888::WHITE);

        assert_eq!(
            matrix.brightness(),
            255,
            r#"initial brightness shall be set to max (255)"#
        );
        matrix.set_brightness(10);
        assert_eq!(matrix.brightness(), 10, r#"brightness shall be set to 10"#);

        matrix.draw_iter(pixels).unwrap();
        matrix.flush().unwrap();

        for i in 0..64 {
            assert_eq!(content[i], RGB8::new(10, 10, 10), r#"expected black pixel"#);
        }
    }

    #[test]
    fn custom_layout() {
        struct CustomLayout;

        /// Custom layout with a different number of LEDs per row.
        ///
        /// # LED indices:
        /// ```text
        /// 0 1 2
        /// 3 4 5 6
        /// 7 8 9 10 11
        /// ```
        impl Layout for CustomLayout {
            fn map(&self, p: Point) -> Option<usize> {
                const LED_PER_ROW: [u8; 3] = [3, 4, 5];

                if p.y < 0
                    || p.y >= LED_PER_ROW.len() as i32
                    || p.x < 0
                    || p.x >= i32::from(LED_PER_ROW[p.y as usize])
                {
                    return None;
                }

                let mut index = 0;
                for y in 0..p.y as usize {
                    index += usize::from(LED_PER_ROW[y]);
                }
                index += p.x as usize;

                Some(index)
            }

            fn size(&self) -> Size {
                Size::new(5, 3)
            }
        }

        let content = &mut [RGB8::new(0, 0, 0); 3 + 4 + 5];
        let writer = MockWriter { content };
        let mut matrix = SmartLedMatrix::<_, _, { 3 + 4 + 5 }>::new(writer, CustomLayout);

        // draw vertical line of red pixels on the left edge
        let mut bb = matrix.bounding_box();
        bb.size.width = 1;
        matrix
            .draw_iter(bb.points().map(|p| Pixel(p, Rgb888::RED)))
            .unwrap();

        matrix.flush().unwrap();

        const B: RGB8 = RGB8::new(0, 0, 0);
        const R: RGB8 = RGB8::new(255, 0, 0);
        assert_eq!(
            content,
            &[
                R, B, B, //
                R, B, B, B, //
                R, B, B, B, B, //
            ]
        );
    }
}
