//! An embedded-graphics display driver for UEFI systems
#![no_std]
use core::convert::TryInto;
use embedded_graphics::{drawable::Pixel, pixelcolor::*, prelude::*, DrawTarget};
use uefi::proto::console::gop::*;

#[derive(Debug)]
pub struct Unsupported(());

/// UEFI Display driver.
///
/// UEFI supports multiple different pixel formats, but embedded-graphics only
/// supports one.
/// To solve this, this display is generic over `Into<Bgr888>`.
///
/// At the moment this display only supports the BGR888 and RGB888 UEFI pixel
/// formats. BltOnly and Bitmask are unsupported.
pub struct UefiDisplay<'a> {
    info: ModeInfo,
    fb: FrameBuffer<'a>,
}

impl<'a> UefiDisplay<'a> {
    /// Create a new [`UefiDisplay`].
    pub fn new(info: ModeInfo, fb: FrameBuffer<'a>) -> Self {
        Self { info, fb }
    }

    pub fn size(&self) -> Size {
        let (width, height) = self.info.resolution();
        // `as` cast is okay, ModeInfo::resolution casts to usize from u32 for some
        // reason..
        Size::new(width as u32, height as u32)
    }
}

impl<'a, T: Into<Bgr888> + PixelColor> DrawTarget<T> for UefiDisplay<'a> {
    type Error = Unsupported;

    fn draw_pixel(&mut self, item: Pixel<T>) -> Result<(), Self::Error> {
        let Pixel(point, color) = item;
        let mut bytes = [0u8; 3];
        match self.info.pixel_format() {
            PixelFormat::RGB => {
                bytes
                    .copy_from_slice(&Rgb888::from(color.into()).into_storage().to_be_bytes()[1..]);
            }
            PixelFormat::BGR => {
                bytes.copy_from_slice(&color.into().into_storage().to_be_bytes()[1..]);
            }
            _ => return Err(Unsupported(())),
        }
        let Size { width, height } = <Self as DrawTarget<T>>::size(self);
        let stride: u64 = self
            .info
            .stride()
            .try_into()
            .expect("Stride didn't fit in u64. Buggy UEFI firmware?");
        let (x, y) = (point.x as u64, point.y as u64);
        if x < width.into() && y < height.into() {
            let index: usize = (((y * stride) + x) * 4)
                .try_into()
                .expect("Framebuffer index overflowed usize");
            unsafe { self.fb.write_value(index, bytes) };
        }
        Ok(())
    }

    fn size(&self) -> Size {
        self.size()
    }
}
