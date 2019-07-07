pub fn gaussian_blur(data: &mut Vec<[u8;3]>, width: usize, height: usize, blur_radius: f32)
{
    let bxs = create_box_gauss(blur_radius, 3);
    let mut backbuf = data.clone();

    box_blur(&mut backbuf, data, width, height, ((bxs[0] - 1) / 2) as usize);
    box_blur(&mut backbuf, data, width, height, ((bxs[1] - 1) / 2) as usize);
    box_blur(&mut backbuf, data, width, height, ((bxs[2] - 1) / 2) as usize);
}

#[inline]
fn create_box_gauss(sigma: f32, n: usize)
-> Vec<i32>
{
    let n_float = n as f32;

    // Ideal averaging filter width
    let w_ideal = (12.0 * sigma * sigma / n_float).sqrt() + 1.0;
    let mut wl: i32 = w_ideal.floor() as i32;

    if wl % 2 == 0 { wl -= 1; };

    let wu = wl + 2;

    let wl_float = wl as f32;
    let m_ideal = (12.0 * sigma * sigma - n_float * wl_float * wl_float - 4.0 * n_float * wl_float - 3.0 * n_float) /
                  (-4.0 * wl_float - 4.0);
    let m: usize = m_ideal.round() as usize;


    let mut sizes = Vec::<i32>::new();

    for i in 0..n {
        if i < m {
            sizes.push(wl);
        } else {
            sizes.push(wu);
        }
    }

    sizes
}

/// Needs 2x the same image
#[inline]
fn box_blur(backbuf: &mut Vec<[u8;3]>, frontbuf: &mut Vec<[u8;3]>, width: usize, height: usize, blur_radius: usize)
{
    box_blur_horz(backbuf, frontbuf, width, height, blur_radius);
    box_blur_vert(frontbuf, backbuf, width, height, blur_radius);
}

#[inline]
fn box_blur_vert(backbuf: &[[u8;3]], frontbuf: &mut [[u8;3]], width: usize, height: usize, blur_radius: usize)
{
    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for i in 0..width {

        let mut ti: usize = i;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius * width;

        let fv: [u8;3] = backbuf[ti];
        let lv: [u8;3] = backbuf[ti + width * (height - 1)];

        let mut val_r: isize = (blur_radius as isize + 1) * (fv[0] as isize);
        let mut val_g: isize = (blur_radius as isize + 1) * (fv[1] as isize);
        let mut val_b: isize = (blur_radius as isize + 1) * (fv[2] as isize);

        for j in 0..blur_radius {
            let bb = backbuf[ti + j * width];
            val_r += bb[0] as isize;
            val_g += bb[1] as isize;
            val_b += bb[2] as isize;
        }

        for _ in 0..(blur_radius + 1) {
            let bb = backbuf[ri]; ri += width;
            val_r += bb[0] as isize - fv[0] as isize;
            val_g += bb[1] as isize - fv[1] as isize;
            val_b += bb[2] as isize - fv[2] as isize;

            frontbuf[ti] = [round(val_r as f32 * iarr) as u8,
                            round(val_g as f32 * iarr) as u8,
                            round(val_b as f32 * iarr) as u8];
            ti += width;
        }

        for _ in (blur_radius + 1)..(height - blur_radius) {

            let bb1 = backbuf[ri]; ri += width;
            let bb2 = backbuf[li]; li += width;

            val_r += bb1[0] as isize - bb2[0] as isize;
            val_g += bb1[1] as isize - bb2[1] as isize;
            val_b += bb1[2] as isize - bb2[2] as isize;

            frontbuf[ti] = [round(val_r as f32 * iarr) as u8,
                            round(val_g as f32 * iarr) as u8,
                            round(val_b as f32 * iarr) as u8];
            ti += width;
        }

        for _ in (height - blur_radius)..height {
            let bb = backbuf[li]; li += width;

            val_r += lv[0] as isize - bb[0] as isize;
            val_g += lv[1] as isize - bb[1] as isize;
            val_b += lv[2] as isize - bb[2] as isize;

            frontbuf[ti] = [round(val_r as f32 * iarr) as u8,
                            round(val_g as f32 * iarr) as u8,
                            round(val_b as f32 * iarr) as u8];
            ti += width;
        }
    }
}

#[inline]
fn box_blur_horz(backbuf: &[[u8;3]], frontbuf: &mut [[u8;3]], width: usize, height: usize, blur_radius: usize)
{
    assert!(backbuf.len() == frontbuf.len());
    assert!(backbuf.len() == width.checked_mul(height).unwrap());

    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for (backbuf_row, frontbuf_row) in backbuf.chunks_exact(width).zip(frontbuf.chunks_exact_mut(width)) {

        let fv = backbuf_row.first().unwrap(); // first value of the row
        let lv = backbuf_row.last().unwrap();  // last value of the row

        let mut val_r: isize = (blur_radius as isize + 1) * (fv[0] as isize);
        let mut val_g: isize = (blur_radius as isize + 1) * (fv[1] as isize);
        let mut val_b: isize = (blur_radius as isize + 1) * (fv[2] as isize);

        for bb in backbuf_row[0..blur_radius].iter() {
            val_r += bb[0] as isize;
            val_g += bb[1] as isize;
            val_b += bb[2] as isize;
        }

        for (bb, out) in backbuf_row[blur_radius..=blur_radius*2].iter().zip(frontbuf_row.iter_mut()) {
            val_r += bb[0] as isize - fv[0] as isize;
            val_g += bb[1] as isize - fv[1] as isize;
            val_b += bb[2] as isize - fv[2] as isize;

            *out = [round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8];
        }

        for (window, out) in backbuf_row.windows(blur_radius*2+2).zip(frontbuf_row[blur_radius+1..].iter_mut()) {

            let bb1 = window.last().unwrap();
            let bb2 = window.first().unwrap();

            val_r += bb1[0] as isize - bb2[0] as isize;
            val_g += bb1[1] as isize - bb2[1] as isize;
            val_b += bb1[2] as isize - bb2[2] as isize;

            *out = [round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8];
        }

        for (bb, out) in backbuf_row[width-blur_radius..].iter().zip(frontbuf_row[width-blur_radius..].iter_mut()) {

            val_r += lv[0] as isize - bb[0] as isize;
            val_g += lv[1] as isize - bb[1] as isize;
            val_b += lv[2] as isize - bb[2] as isize;

            *out = [round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8];
        }
    }
}

#[inline]
/// Fast rounding for x <= 2^23.
/// This is orders of magnitude faster than built-in rounding intrinsic.
/// 
/// Source: https://stackoverflow.com/a/42386149/585725
fn round(mut x: f32) -> f32 {
    x += 12582912.0;
    x -= 12582912.0;
    x
}