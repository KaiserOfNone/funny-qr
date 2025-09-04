use image::{GrayImage, Luma, RgbaImage};
use qrcode_generator::QrCodeEcc;

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

fn main() {
    let sheet = image::open("sheet.png").unwrap().into_rgba8();

    let result: Vec<Vec<bool>> =
        qrcode_generator::to_matrix("Some other thing", QrCodeEcc::Quartile).unwrap();
    let resolution = result.len() as u32 * 32;

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
    copy_paste(&sheet, &mut result_image, 0, 0, 32, 32, 0, 0);
    result_image.save("test.png").unwrap();
}
