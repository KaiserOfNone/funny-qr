use image::{Rgba, RgbaImage};
use qrcode_generator::QrCodeEcc;

/*
* TODO list:
 * - Add a basic tiler for regular ass QRs
 * - CLI calling for non hardcoded shit
 * - Add the alignment calculator function
 * - Add a "cleanup" step to add the alignment markers
*/

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

fn main() {
    let sheet = image::open("sheet.png").unwrap().into_rgba8();

    let result: Vec<Vec<bool>> = qrcode_generator::to_matrix("Lucas", QrCodeEcc::Quartile).unwrap();
    let mut tiler = BaseTiler::new(32, result.len() as u32);

    for (y, row) in result.iter().enumerate() {
        for (x, val) in row.iter().enumerate() {
            tiler.tile(x as u32, y as u32, *val);
        }
    }
    tiler.finalize().save("test.png").unwrap();
    /*
    let mut result_image = RgbaImage::new(resolution, resolution);
    let mut bouncer = false;
    for (y, row) in result.iter().enumerate() {
        for (x, val) in row.iter().enumerate() {
            let mut x0 = if bouncer { 32 } else { 0 };
            x0 = if *val { x0 + 64 } else { x0 };
            let dst_x = (x * 32) as u32;
            let dst_y = (y * 32) as u32;
            copy_paste(&sheet, &mut result_image, x0, 0, 32, 32, dst_x, dst_y);
            bouncer = !bouncer;
        }
    }
    result_image.save("test.png").unwrap();
    */
}
