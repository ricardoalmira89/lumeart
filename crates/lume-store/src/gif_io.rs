//! GIF animado de una acción (exportar animación), con el crate `gif`
//! (Rust puro): cada fotograma con su duración y transparencia de 1 bit.

use lume_domain::anim::Image;
use lume_domain::{DomainError, ErrorCode};

fn gif_error(err: impl std::fmt::Display) -> DomainError {
    DomainError::new(ErrorCode::Io, format!("gif: {err}"))
}

/// Los bytes de un GIF con estos fotogramas (todos del tamaño del primero) y
/// sus duraciones en milisegundos (el GIF las guarda en centésimas).
pub fn encode_gif(
    frames: &[Image],
    durations_ms: &[u32],
    looping: bool,
) -> Result<Vec<u8>, DomainError> {
    let Some(first) = frames.first() else {
        return Err(DomainError::invalid_args("gif: no hay fotogramas"));
    };
    let (w, h) = first.size();
    let (w16, h16) = (
        u16::try_from(w).map_err(|_| gif_error("demasiado ancho"))?,
        u16::try_from(h).map_err(|_| gif_error("demasiado alto"))?,
    );
    let mut out = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut out, w16, h16, &[]).map_err(gif_error)?;
        encoder
            .set_repeat(if looping {
                gif::Repeat::Infinite
            } else {
                gif::Repeat::Finite(0)
            })
            .map_err(gif_error)?;
        for (i, img) in frames.iter().enumerate() {
            if img.size() != (w, h) {
                return Err(gif_error("los fotogramas tienen tamaños distintos"));
            }
            let mut rgba: Vec<u8> = img
                .pixels()
                .iter()
                .flat_map(|c| {
                    // Transparencia de 1 bit: lo casi invisible, fuera.
                    if c.a < 128 {
                        [0, 0, 0, 0]
                    } else {
                        [c.r, c.g, c.b, 255]
                    }
                })
                .collect();
            let mut frame = gif::Frame::from_rgba_speed(w16, h16, &mut rgba, 10);
            let ms = durations_ms.get(i).copied().unwrap_or(100);
            frame.delay = u16::try_from(ms.div_ceil(10).max(1)).unwrap_or(u16::MAX);
            frame.dispose = gif::DisposalMethod::Background;
            encoder.write_frame(&frame).map_err(gif_error)?;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_domain::Rgba;

    #[test]
    fn frames_and_delays_survive() {
        let mut a = Image::new(4, 3);
        a.set(1, 1, Rgba::rgb(255, 0, 0));
        let mut b = Image::new(4, 3);
        b.set(2, 1, Rgba::rgb(0, 0, 255));
        let bytes = encode_gif(&[a, b], &[100, 250], true).unwrap();
        assert!(bytes.starts_with(b"GIF89a"));
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options.read_info(bytes.as_slice()).unwrap();
        assert_eq!((decoder.width(), decoder.height()), (4, 3));
        let mut delays = Vec::new();
        let mut reds = Vec::new();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            delays.push(frame.delay);
            let px = |x: usize, y: usize| &frame.buffer[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4];
            reds.push(px(1, 1).to_vec());
            assert_eq!(px(0, 0)[3], 0, "fondo transparente");
        }
        assert_eq!(delays, [10, 25]);
        assert_eq!(reds[0], [255, 0, 0, 255]);
        assert!(encode_gif(&[], &[], true).is_err());
    }
}
