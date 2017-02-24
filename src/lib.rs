#![crate_name="orbimage"]
#![crate_type="lib"]

extern crate orbclient;
extern crate resize;

use std::{cmp, slice};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use orbclient::{Color, Renderer};

pub use bmp::parse as parse_bmp;
pub use jpg::parse as parse_jpg;
pub use png::parse as parse_png;
pub use resize::Type as ResizeType;

pub mod bmp;
pub mod jpg;
pub mod png;

pub struct ImageRoi<'a> {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    image: &'a Image
}

impl<'a> ImageRoi<'a> {
    /// Draw the ROI on a window
    pub fn draw<R: Renderer>(&self, renderer: &mut R, x: i32, mut y: i32) {
        let stride = self.image.w;
        let mut offset = (self.y * stride + self.x) as usize;
        let last_offset = cmp::min(((self.y + self.h) * stride + self.x) as usize, self.image.data.len());
        while offset < last_offset {
            let next_offset = offset + stride as usize;
            renderer.image(x, y, self.w, 1, &self.image.data[offset..]);
            offset = next_offset;
            y += 1;
        }
    }
}

pub struct Image {
    w: u32,
    h: u32,
    data: Box<[Color]>
}

impl Image {
    /// Create a new image
    pub fn new(width: u32, height: u32) -> Self {
        Self::from_color(width, height, Color::rgb(0, 0, 0))
    }

    /// Create a new image filled whole with color
    pub fn from_color(width: u32, height: u32, color: Color) -> Self {
        Self::from_data(width, height, vec![color; width as usize * height as usize].into_boxed_slice()).unwrap()
    }

    /// Create a new image from a boxed slice of colors
    pub fn from_data(width: u32, height: u32, data: Box<[Color]>) -> Result<Self, String> {
        if (width * height) as usize != data.len() {
            return Err("not enough or too much data given compared to width and height".to_string())
        }

        Ok(Image {
            w: width,
            h: height,
            data: data,
        })
    }

    /// Load an image from file path. Supports BMP and PNG
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = try!(File::open(&path).map_err(|err| format!("failed to open image: {}", err)));
        let mut data: Vec<u8> = Vec::new();
        let _ = try!(file.read_to_end(&mut data).map_err(|err| format!("failed to read image: {}", err)));
        //TODO: Use magic to match file instead of extension
        match path.as_ref().extension() {
            Some(extension_os) => match extension_os.to_str() {
                Some(extension) => match extension.to_lowercase().as_str() {
                    "bmp" => parse_bmp(&data),
                    "jpg" | "jpeg" => parse_jpg(&data),
                    "png" => parse_png(&data),
                    other => Err(format!("unknown image extension: {}", other))
                },
                None => Err("image extension not valid unicode".to_string())
            },
            None => Err("no image extension".to_string())
        }
    }

    /// Create a new empty image
    pub fn default() -> Self {
        Self::new(0, 0)
    }

    // Get a resized version of the image
    pub fn resize(&self, w: u32, h: u32, resize_type: ResizeType) -> Result<Self, String> {
        let mut dst_color = vec![Color { data: 0 }; w as usize * h as usize].into_boxed_slice();

        let src = unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4)
        };

        let mut dst = unsafe {
            slice::from_raw_parts_mut(dst_color.as_mut_ptr() as *mut u8, dst_color.len() * 4)
        };

        let mut resizer = resize::new(self.w as usize, self.h as usize,
                                      w as usize, h as usize,
                                      resize::Pixel::RGBA, resize_type);
        resizer.resize(&src, &mut dst);

        Image::from_data(w, h, dst_color)
    }

    /// Get a piece of the image
    pub fn roi<'a>(&'a self, x: u32, y: u32, w: u32, h: u32) -> ImageRoi<'a> {
        let x1 = cmp::min(x, self.width());
        let y1 = cmp::min(y, self.height());
        let x2 = cmp::max(x1, cmp::min(x + w, self.width()));
        let y2 = cmp::max(y1, cmp::min(y + h, self.height()));

        ImageRoi {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1,
            image: self
        }
    }

    /// Return a boxed slice of colors making up the image
    pub fn into_data(self) -> Box<[Color]> {
        self.data
    }

    /// Draw the image on a window
    pub fn draw<R: Renderer>(&self, renderer: &mut R, x: i32, y: i32) {
        renderer.image(x, y, self.w, self.h, &self.data);
    }
}

impl Renderer for Image {
    /// Get the width of the image in pixels
    fn width(&self) -> u32 {
        self.w
    }

    /// Get the height of the image in pixels
    fn height(&self) -> u32 {
        self.h
    }

    /// Return a reference to a slice of colors making up the image
    fn data(&self) -> &[Color] {
        &self.data
    }

    /// Return a mutable reference to a slice of colors making up the image
    fn data_mut(&mut self) -> &mut [Color] {
        &mut self.data
    }

    fn sync(&mut self) -> bool {
        true
    }
}
