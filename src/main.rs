use clap::{Parser, ValueEnum};
use image::{Rgba, RgbaImage};
use qrcode_generator::QrCodeEcc;

/*
* TODO list:
 * - Add the alignment calculator function
 * - Add a "cleanup" step to add the alignment markers
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Tilers {
    Base,
    Bouncy,
}

impl ToString for Tilers {
    fn to_string(&self) -> String {
        match self {
            Tilers::Base => "base".to_owned(),
            Tilers::Bouncy => "bounce".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Parser)]
struct Args {
    /// Output file name
    #[arg(short, long)]
    target: Option<String>,

    /// Tiler style: base, bounce (ignores block size)
    #[arg(short, long, value_enum, default_value_t = Tilers::Base)]
    style: Tilers,

    /// Block size in pixels
    #[arg(short, long, default_value_t = 32)]
    block_size: u32,

    /// Content to encode (must be last)
    #[arg(required = true)]
    content: String,
}

fn copy_paste(
    src: &RgbaImage,
    dst: &mut RgbaImage,
    srcx0: u32,
    srcy0: u32,
    srcwidth: u32,
    srcheight: u32,
    dstx0: u32,
    dsty0: u32,
) {
    for x in 0..srcwidth {
        for y in 0..srcheight {
            let srcx = srcx0 + x;
            let srcy = srcy0 + y;
            let dstx = dstx0 + x;
            let dsty = dsty0 + y;
            let pixel = src.get_pixel(srcx, srcy);
            let dst_pixel = dst.get_pixel_mut(dstx, dsty);
            dst_pixel.clone_from(pixel);
        }
    }
}

trait Tiler {
    fn tile(&mut self, x: u32, y: u32, value: bool);
    fn finalize(&self) -> RgbaImage;
}

struct BaseTiler {
    block_size: u32,
    image: RgbaImage,
    black: RgbaImage,
    white: RgbaImage,
}

impl BaseTiler {
    fn new(block_size: u32, width: u32) -> BaseTiler {
        let image = RgbaImage::new(width * block_size, width * block_size);
        let black = RgbaImage::from_pixel(block_size, block_size, Rgba([0, 0, 0, 255]));
        let white = RgbaImage::from_pixel(block_size, block_size, Rgba([255, 255, 255, 255]));
        BaseTiler {
            block_size,
            image,
            black,
            white,
        }
    }
}

impl Tiler for BaseTiler {
    fn tile(&mut self, x: u32, y: u32, value: bool) {
        let dstx = x * self.block_size;
        let dsty = y * self.block_size;
        let src = if value { &self.black } else { &self.white };
        copy_paste(
            src,
            &mut self.image,
            0,
            0,
            self.block_size,
            self.block_size,
            dstx,
            dsty,
        );
    }

    fn finalize(&self) -> RgbaImage {
        self.image.clone()
    }
}

struct BouncyTiler {
    block_size: u32,
    image: RgbaImage,
    sheet: RgbaImage,
    bounce: bool,
}

impl BouncyTiler {
    fn new(width: u32) -> BouncyTiler {
        let sheet = image::open("sheet.png").unwrap().into_rgba8();
        let block_size = 32; // Constant from the sprite I'm using
        BouncyTiler {
            block_size,
            image: RgbaImage::new(width * block_size, width * block_size),
            sheet,
            bounce: false,
        }
    }
}

impl Tiler for BouncyTiler {
    fn tile(&mut self, x: u32, y: u32, value: bool) {
        let mut x0 = if self.bounce { 32 } else { 0 };
        x0 = if value { x0 + 64 } else { x0 };
        let dst_x = x * self.block_size;
        let dst_y = y * self.block_size;
        copy_paste(
            &self.sheet,
            &mut self.image,
            x0,
            0,
            self.block_size,
            self.block_size,
            dst_x,
            dst_y,
        );
        self.bounce = !self.bounce;
    }

    fn finalize(&self) -> RgbaImage {
        self.image.clone()
    }
}

fn main() {
    let args = Args::parse();
    let target = args.target.unwrap_or("out.png".to_owned());
    let result: Vec<Vec<bool>> =
        qrcode_generator::to_matrix(args.content, QrCodeEcc::Quartile).unwrap();
    let width = result.len() as u32;

    let mut tiler: Box<dyn Tiler> = match args.style {
        Tilers::Base => Box::new(BaseTiler::new(args.block_size, width)),
        Tilers::Bouncy => Box::new(BouncyTiler::new(width)),
    };

    for (y, row) in result.iter().enumerate() {
        for (x, val) in row.iter().enumerate() {
            tiler.tile(x as u32, y as u32, *val);
        }
    }

    tiler.finalize().save(target).unwrap();
}
