use clap::{Parser, ValueEnum};
use image::{Rgba, RgbaImage};
use qrcode_generator::QrCodeEcc;

/*
* TODO list:
 * - Add a "Stars" pattern
 * - Think of other patterns
 * - Embed the textures into the binary
 * - Separate the code into a library
 * - Create wasm bindings
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

fn create_position_block(block_size: u32) -> RgbaImage {
    let black = RgbaImage::from_pixel(block_size, block_size, Rgba([0, 0, 0, 255]));
    let white = RgbaImage::from_pixel(block_size, block_size, Rgba([255, 255, 255, 255]));
    let mut position_block = RgbaImage::new(7 * block_size, 7 * block_size);

    // Outer black square (7x7)
    for y in 0..7 {
        for x in 0..7 {
            copy_paste(
                &black,
                &mut position_block,
                0,
                0,
                block_size,
                block_size,
                x * block_size,
                y * block_size,
            );
        }
    }
    // Inner white square (5x5)
    for y in 1..6 {
        for x in 1..6 {
            copy_paste(
                &white,
                &mut position_block,
                0,
                0,
                block_size,
                block_size,
                x * block_size,
                y * block_size,
            );
        }
    }
    // Center black square (3x3)
    for y in 2..5 {
        for x in 2..5 {
            copy_paste(
                &black,
                &mut position_block,
                0,
                0,
                block_size,
                block_size,
                x * block_size,
                y * block_size,
            );
        }
    }
    position_block
}

fn create_alignment_block(block_size: u32) -> RgbaImage {
    let black = RgbaImage::from_pixel(block_size, block_size, Rgba([0, 0, 0, 255]));
    let white = RgbaImage::from_pixel(block_size, block_size, Rgba([255, 255, 255, 255]));
    let mut alignment_block = RgbaImage::new(5 * block_size, 5 * block_size);

    // Outer black square (5x5)
    for y in 0..5 {
        for x in 0..5 {
            copy_paste(
                &black,
                &mut alignment_block,
                0,
                0,
                block_size,
                block_size,
                x * block_size,
                y * block_size,
            );
        }
    }
    // Inner white square (3x3)
    for y in 1..4 {
        for x in 1..4 {
            copy_paste(
                &white,
                &mut alignment_block,
                0,
                0,
                block_size,
                block_size,
                x * block_size,
                y * block_size,
            );
        }
    }
    // Center black pixel (1x1)
    copy_paste(
        &black,
        &mut alignment_block,
        0,
        0,
        block_size,
        block_size,
        2 * block_size,
        2 * block_size,
    );

    alignment_block
}

fn write_position_blocks(image: &mut RgbaImage, block_size: u32) {
    let positions = [
        (0, 0),                               // Top-left
        (image.width() - 7 * block_size, 0),  // Top-right
        (0, image.height() - 7 * block_size), // Bottom-left
    ];
    let position_block = create_position_block(block_size);
    for &(px, py) in &positions {
        copy_paste(
            &position_block,
            image,
            0,
            0,
            7 * block_size,
            7 * block_size,
            px,
            py,
        );
    }
}

fn alignment_coord_list(version: u32, size: u32) -> Vec<u32> {
    if version == 1 {
        return vec![];
    }
    let divs = 2 + version / 7;
    let total_dist = size - 7 - 6;
    let divisor = 2 * (divs - 1);
    // Step must be even, for alignment patterns to agree with timing patterns
    let step = ((total_dist + divisor / 2 + 1) / divisor) * 2;
    let mut coords = vec![6];
    for i in (0..=(divs - 2)).rev() {
        coords.push(size - 7 - i * step);
    }
    coords
}

fn write_alignment_blocks(image: &mut RgbaImage, block_size: u32) {
    let version = (image.width() / block_size - 17) / 4;
    let size = image.width();
    let data_size = size / block_size;
    let coords = alignment_coord_list(version, data_size);
    let alignment_block = create_alignment_block(block_size);

    for &x in &coords {
        for &y in &coords {
            // Skip the corners (where position blocks are)
            if (x == 6 && y == 6)
                || (x == 6 && y == data_size - 7)
                || (x == data_size - 7 && y == 6)
            {
                continue;
            }
            let top_left_x = x * block_size - (4 * block_size) / 2;
            let top_left_y = y * block_size - (4 * block_size) / 2;
            copy_paste(
                &alignment_block,
                image,
                0,
                0,
                5 * block_size,
                5 * block_size,
                top_left_x,
                top_left_y,
            );
        }
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
    let mut image = tiler.finalize();
    if args.style != Tilers::Base {
        write_position_blocks(&mut image, args.block_size);
        write_alignment_blocks(&mut image, args.block_size);
    }
    image.save(target).unwrap();
}
