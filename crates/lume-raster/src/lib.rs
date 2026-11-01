//! Adaptador de rasterizado de LumeArt.
//!
//! Implementa el puerto `Rasterizer` con `resvg` + `tiny-skia`: convierte SVG
//! en imagen sin Chrome, con opciones de ancho, rejilla de coordenadas,
//! recorte, superposición y fondo (las mismas que `RasterOptions` en pxcli).
//! El encuadre y el SVG compuesto salen de `lume_domain::vec::raster`; aquí
//! solo se pinta.
//!
//! Además, [`trace`] calca imágenes en trazos (con el motor de vtracer) y
//! [`pdf`] guarda un SVG como PDF con capas (con `svg2pdf`).
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

pub mod hits;
pub mod pdf;
pub mod trace;

use std::sync::{Arc, OnceLock};

use lume_app::ports::{PortError, RasterImage, RasterOptions, Rasterizer};
use lume_domain::Rgba;
use lume_domain::vec::raster::{compose, frame};
use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg::{self, ImageHrefResolver, fontdb};

/// Fuentes de respaldo, por si el sistema no tiene ninguna de las de cada
/// familia genérica (las mismas de la interfaz).
pub(crate) const GEIST: &[u8] = include_bytes!("../../../assets/fonts/Geist-Regular.ttf");
pub(crate) const GEIST_MONO: &[u8] = include_bytes!("../../../assets/fonts/GeistMono-Regular.ttf");

/// Familias genéricas, en el orden en que se buscan entre las del sistema.
/// Como Chrome: `serif` es Times New Roman y `sans-serif` Arial, o las que
/// las sustituyen con las mismas medidas (Liberation, Tinos/Arimo, Nimbus).
/// La última es la de respaldo, que siempre está.
pub(crate) const SERIF: &[&str] = &[
    "Times New Roman",
    "Liberation Serif",
    "Tinos",
    "Nimbus Roman",
    "Times",
    "DejaVu Serif",
    "Noto Serif",
    "Georgia",
    "Geist",
];
pub(crate) const SANS_SERIF: &[&str] = &[
    "Arial",
    "Liberation Sans",
    "Arimo",
    "Nimbus Sans",
    "Helvetica",
    "DejaVu Sans",
    "Noto Sans",
    "Geist",
];
pub(crate) const MONOSPACE: &[&str] = &[
    "DejaVu Sans Mono",
    "Noto Sans Mono",
    "Liberation Mono",
    "Courier New",
    "Menlo",
    "Consolas",
    "Geist Mono",
];

/// Tamaño de letra de un texto sin `font-size`: el de una página de Chrome.
const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Las fuentes: las del sistema y las de respaldo, cargadas una sola vez.
fn fonts() -> Arc<fontdb::Database> {
    static FONTS: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            db.load_font_data(GEIST.to_vec());
            db.load_font_data(GEIST_MONO.to_vec());
            let serif = first_present(&db, SERIF);
            db.set_serif_family(serif);
            db.set_sans_serif_family(first_present(&db, SANS_SERIF));
            db.set_monospace_family(first_present(&db, MONOSPACE));
            db.set_cursive_family(serif);
            db.set_fantasy_family(serif);
            Arc::new(db)
        })
        .clone()
}

fn first_present(db: &fontdb::Database, names: &'static [&'static str]) -> &'static str {
    names
        .iter()
        .copied()
        .find(|name| {
            db.faces()
                .any(|face| face.families.iter().any(|(family, _)| family == name))
        })
        .unwrap_or(names[names.len() - 1])
}

/// Nombre de la familia que hace de `serif` (la de un texto sin
/// `font-family`).
pub fn serif_family() -> String {
    fonts().family_name(&fontdb::Family::Serif).to_string()
}

/// `Rasterizer` con resvg.
#[derive(Debug, Default, Clone, Copy)]
pub struct ResvgRasterizer;

impl ResvgRasterizer {
    pub fn new() -> Self {
        Self
    }
}

impl Rasterizer for ResvgRasterizer {
    fn rasterize(
        &self,
        svg: &str,
        width: f64,
        height: f64,
        options: &RasterOptions,
    ) -> Result<RasterImage, PortError> {
        let frame = frame(width, height, options.crop, options.width)
            .map_err(|e| PortError::Invalid(e.message))?;
        let document = compose(svg, &frame, options.grid, options.overlay.as_deref());
        render(&document, frame.width, frame.height, options.background)
    }
}

/// Las opciones de usvg: las fuentes, el tamaño de letra de Chrome y solo
/// imágenes incrustadas (`data:`): un SVG no lee archivos.
pub(crate) fn options() -> usvg::Options<'static> {
    let fonts = fonts();
    usvg::Options {
        font_family: fonts.family_name(&fontdb::Family::Serif).to_string(),
        font_size: DEFAULT_FONT_SIZE,
        fontdb: fonts,
        image_href_resolver: ImageHrefResolver {
            resolve_data: ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    }
}

/// Pinta un SVG ya compuesto en una imagen de `width` × `height` sobre
/// `background` (transparente si no hay).
pub fn render(
    svg: &str,
    width: u32,
    height: u32,
    background: Option<Rgba>,
) -> Result<RasterImage, PortError> {
    let options = options();
    let tree = usvg::Tree::from_str(svg, &options)
        .map_err(|e| PortError::Invalid(format!("SVG no válido: {e}")))?;
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| {
        PortError::Invalid(format!("tamaño de salida no válido: {width}x{height}"))
    })?;
    if let Some(c) = background {
        pixmap.fill(Color::from_rgba8(c.r, c.g, c.b, c.a));
    }
    // El SVG compuesto mide lo mismo que la imagen: sin transformar.
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Ok(RasterImage {
        width,
        height,
        rgba,
    })
}
