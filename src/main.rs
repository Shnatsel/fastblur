#![feature(test)]

mod bench;
pub mod blur;

extern crate x11_dl;

fn main() {
    let display = x11_get_display();
    let (mut data, width, height) = x11_make_screenshot(display, 0, 0, None, None);
    blur::gaussian_blur(&mut data, width as usize, height as usize, 10.0);
    write_image("new.ppm", &data, width, height).unwrap();
}

#[cfg(target_os = "linux")]
fn x11_get_display<'a>() -> &'a mut x11_dl::xlib::Display {

    let xlib = match x11_dl::xlib::Xlib::open() {
        Ok(x) => x,
        Err(xerr) => panic!("Error: {}", xerr.detail()),
    };

    let dpy = unsafe { (xlib.XOpenDisplay)(&0) };

    if dpy.is_null() {
        panic!("Error opening connection to X Server!");
    } else {
        unsafe { &mut *dpy }
    }
}

#[cfg(target_os = "linux")]
fn x11_make_screenshot(display: &mut x11_dl::xlib::Display, offset_x: i32, offset_y: i32, width: Option<i32>, height: Option<i32>)
-> (Vec<[u8; 3]>, u32, u32)
{
    let xlib = match x11_dl::xlib::Xlib::open() {
        Ok(x) => x,
        Err(xerr) => panic!("Error: {}", xerr.detail()),
    };

    let root = unsafe { (xlib.XDefaultRootWindow)(display) };
    let mut gwa: x11_dl::xlib::XWindowAttributes = unsafe { ::std::mem::zeroed() };
    unsafe { (xlib.XGetWindowAttributes)(display, root, &mut gwa) };

    let width = width.unwrap_or(gwa.width);
    let height = height.unwrap_or(gwa.height);

    let image_raw = unsafe { (xlib.XGetImage)(display, root, 0, 0, width as u32, height as u32, (xlib.XAllPlanes)(), x11_dl::xlib::ZPixmap) };

    let image = {
        if image_raw.is_null() {
            panic!("Error getting image!");
        } else {
            unsafe { &mut *image_raw }
        }
    };

    // todo: check if 3 is the correct number
    let capacity = (width * height) as usize;

    let mut screenshot: Vec<[u8; 3]> = Vec::with_capacity(capacity);
    screenshot.resize(capacity, [0, 0, 0]);

    let red_mask = image.red_mask;
    let green_mask = image.green_mask;
    let blue_mask = image.blue_mask;

    for y in offset_y..height {
        for x in offset_x..width {
            let pixel = unsafe { (xlib.XGetPixel)(image,x,y) };

            let blue  = (pixel & blue_mask) as u8;
            let green = ((pixel & green_mask) >> 8) as u8;
            let red   = ((pixel & red_mask)   >> 16) as u8;

            screenshot[((width * y) + x) as usize] = [red, green, blue];
/*
            screenshot[((x + width * y) * 3)      as usize] = red;
            screenshot[((x + width * y) * 3 + 1)  as usize] = green;
            screenshot[((x + width * y) * 3 + 2)  as usize] = blue;
*/
        }
    }

    (screenshot, width as u32, height as u32)
}

fn write_image<S>(filename: S, data: &[[u8;3]], width: u32, height: u32)
-> Result<(), ::std::io::Error> where S: Into<String>
{
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;

    let mut file = BufWriter::new(File::create(filename.into())?);
    let header = format!("P6\n{}\n{}\n{}\n", width, height, 255);

    file.write(header.as_bytes())?;

    for px in data {
        file.write(px)?;
    }

    Ok(())
}



