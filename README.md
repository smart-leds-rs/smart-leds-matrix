# This is a very WorkInProgress project.

## Smart LEDs matrix
A `DrawTarget` implementation to use (one, or more) smart LED matrixes as a graphics display driven by [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) `Drawable` objects.
The integrated driver is from [smart-leds](https://docs.rs/smart-leds/latest/smart_leds/) crate.

# Plan
* Add matrix layout parameter / plugin option, which would be able to adapt multiple type and instances of matrixes as one single display (like 4 8x8 matrixes in a grid to have 16x16 or 8x32 display as a result).

# Status
It works on some level. Rectangles are fine.

There are interesting issues though, with my setup (stm32f401 + 8x8 ws2812 matrix):
* circles are not exacly drawn always to the same position
* write operation usually gets back with an overrun error, while the display is still updated
