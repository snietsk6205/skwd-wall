//! R1 (custom build): decodes the Settings Home background once per change:
//! a sharp copy (at most 1600 px wide) and a small blurred copy for the top
//! bar. Runs on its own thread; the result comes back as a `Wake`.

use futures_channel::mpsc::UnboundedSender;

use crate::infrastructure::runtime::Wake;

/// RGBA pixels with their size.
pub type Pixels = (Vec<u8>, u32, u32);

fn decode_home_backdrop(path: &str) -> Option<(Pixels, Pixels)> {
    let picture = image::open(path).ok()?;
    let sharp = if picture.width() > 1600 {
        picture.resize(1600, 1600, image::imageops::FilterType::Triangle)
    } else {
        picture
    };
    let small = sharp.resize(320, 320, image::imageops::FilterType::Triangle).to_rgba8();
    let blurred = image::imageops::fast_blur(&small, 6.0);
    let sharp = sharp.to_rgba8();
    let (sw, sh) = sharp.dimensions();
    let (bw, bh) = blurred.dimensions();
    Some(((sharp.into_raw(), sw, sh), (blurred.into_raw(), bw, bh)))
}

pub fn spawn_home_backdrop(source: String, poster: String, sender: UnboundedSender<Wake>) {
    let _ = std::thread::Builder::new().name("home-backdrop".into()).spawn(move || {
        let decoded = decode_home_backdrop(&poster);
        let _ =
            sender.unbounded_send(Wake::HomeBackdrop { source, decoded: decoded.map(Box::new) });
    });
}
