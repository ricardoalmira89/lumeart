//! El dibujo como PDF con capas (para `.pdf` y `.ai`, que Illustrator abre
//! como PDF), con `svg2pdf` en vez de VectorCraft.
//!
//! Cada grupo de primer nivel del SVG limpio (el lienzo, cada `layer()`, las
//! formas sueltas) es una capa con su nombre: un grupo de contenido opcional
//! (OCG) del PDF. Cada capa se convierte por separado en un XObject (con las
//! definiciones del dibujo) y la página los dibuja en orden, cada uno dentro
//! de su capa.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use lume_app::ports::PortError;
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, TextStr};
use resvg::usvg::roxmltree;
use svg2pdf::usvg as pdf_usvg;

/// Una capa: su nombre y su SVG (las definiciones y el grupo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub name: String,
    pub svg: String,
}

/// Parte el SVG en capas: una por grupo de primer nivel (con su
/// `data-name`, su `id` o «capa N»). Lo que no es un grupo ni definiciones
/// va en una capa «formas».
pub fn split_layers(svg: &str) -> Result<Vec<Layer>, PortError> {
    let doc = roxmltree::Document::parse(svg)
        .map_err(|e| PortError::Invalid(format!("SVG no válido: {e}")))?;
    let root = doc.root_element();
    let range = root.range();
    let open_end = svg[range.start..]
        .find('>')
        .map(|i| range.start + i + 1)
        .ok_or_else(|| PortError::Invalid("SVG no válido".into()))?;
    let open = &svg[range.start..open_end];
    let mut defs = String::new();
    let mut layers: Vec<Layer> = Vec::new();
    let mut loose = String::new();
    for child in root.children().filter(|n| n.is_element()) {
        let text = &svg[child.range()];
        match child.tag_name().name() {
            "defs" | "style" => defs.push_str(text),
            "g" => {
                let name = child
                    .attribute("data-name")
                    .or_else(|| child.attribute("id"))
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("capa {}", layers.len() + 1));
                layers.push(Layer {
                    name,
                    svg: text.to_string(),
                });
            }
            _ => loose.push_str(text),
        }
    }
    if !loose.is_empty() {
        layers.push(Layer {
            name: "formas".into(),
            svg: loose,
        });
    }
    Ok(layers
        .into_iter()
        .map(|l| Layer {
            svg: format!("{open}{defs}{}</svg>", l.svg),
            name: l.name,
        })
        .collect())
}

/// Las fuentes de `svg2pdf` (su `usvg` es otra versión, con su propia
/// base): las del sistema y Geist de respaldo.
fn fonts() -> Arc<pdf_usvg::fontdb::Database> {
    static FONTS: OnceLock<Arc<pdf_usvg::fontdb::Database>> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut db = pdf_usvg::fontdb::Database::new();
            db.load_system_fonts();
            db.load_font_data(crate::GEIST.to_vec());
            db.load_font_data(crate::GEIST_MONO.to_vec());
            let pick = |db: &pdf_usvg::fontdb::Database, names: &'static [&'static str]| {
                names
                    .iter()
                    .copied()
                    .find(|name| {
                        db.faces()
                            .any(|f| f.families.iter().any(|(family, _)| family == name))
                    })
                    .unwrap_or(names[names.len() - 1])
            };
            let serif = pick(&db, crate::SERIF);
            db.set_serif_family(serif);
            db.set_sans_serif_family(pick(&db, crate::SANS_SERIF));
            db.set_monospace_family(pick(&db, crate::MONOSPACE));
            Arc::new(db)
        })
        .clone()
}

fn pdf_error(e: impl std::fmt::Display) -> PortError {
    PortError::Invalid(format!("no se pudo convertir a PDF: {e}"))
}

/// El SVG como PDF de una página, con una capa por grupo de primer nivel.
pub fn svg_to_pdf(svg: &str) -> Result<Vec<u8>, PortError> {
    let layers = split_layers(svg)?;
    let options = pdf_usvg::Options {
        fontdb: fonts(),
        image_href_resolver: pdf_usvg::ImageHrefResolver {
            resolve_data: pdf_usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..pdf_usvg::Options::default()
    };
    // El tamaño de la página: el del dibujo entero (1 unidad = 1 punto).
    let whole = pdf_usvg::Tree::from_str(svg, &options).map_err(pdf_error)?;
    let (w, h) = (whole.size().width(), whole.size().height());

    let mut alloc = Ref::new(1);
    let catalog_id = alloc.bump();
    let pages_id = alloc.bump();
    let page_id = alloc.bump();
    let content_id = alloc.bump();
    let mut pdf = Pdf::new();
    let mut placed = Vec::new();
    for (i, layer) in layers.iter().enumerate() {
        let tree = pdf_usvg::Tree::from_str(&layer.svg, &options).map_err(pdf_error)?;
        let (chunk, x_ref) =
            svg2pdf::to_chunk(&tree, svg2pdf::ConversionOptions::default()).map_err(pdf_error)?;
        let mut map = HashMap::new();
        let chunk = chunk.renumber(|old| *map.entry(old).or_insert_with(|| alloc.bump()));
        let x_ref = map[&x_ref];
        let ocg = alloc.bump();
        pdf.extend(&chunk);
        placed.push((format!("X{i}"), format!("oc{i}"), x_ref, ocg, &layer.name));
    }
    let ocgs: Vec<Ref> = placed.iter().map(|p| p.3).collect();
    {
        let mut catalog = pdf.catalog(catalog_id);
        catalog.pages(pages_id);
        let mut props = catalog.insert(Name(b"OCProperties")).dict();
        props
            .insert(Name(b"OCGs"))
            .array()
            .items(ocgs.iter().copied());
        let mut d = props.insert(Name(b"D")).dict();
        d.pair(Name(b"Name"), TextStr("Capas"));
        d.pair(Name(b"BaseState"), Name(b"ON"));
        d.insert(Name(b"Order")).array().items(ocgs.iter().copied());
        d.insert(Name(b"ON")).array().items(ocgs.iter().copied());
        d.finish();
        props.finish();
    }
    pdf.pages(pages_id).kids([page_id]).count(1);
    {
        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, w, h));
        page.parent(pages_id);
        page.contents(content_id);
        let mut resources = page.resources();
        {
            let mut xobjects = resources.x_objects();
            for (x, _, x_ref, _, _) in &placed {
                xobjects.pair(Name(x.as_bytes()), *x_ref);
            }
        }
        {
            // Cada nombre de capa del contenido, con su OCG.
            let mut properties = resources.insert(Name(b"Properties")).dict();
            for (_, oc, _, ocg, _) in &placed {
                properties.pair(Name(oc.as_bytes()), *ocg);
            }
        }
    }
    for (_, _, _, ocg, name) in &placed {
        let mut dict = pdf.indirect(*ocg).dict();
        dict.pair(Name(b"Type"), Name(b"OCG"));
        dict.pair(Name(b"Name"), TextStr(name));
    }
    let mut content = Content::new();
    for (x, oc, _, _, _) in &placed {
        content
            .begin_marked_content_with_properties(Name(b"OC"))
            .properties_named(Name(oc.as_bytes()));
        content.save_state();
        content.transform([w, 0.0, 0.0, h, 0.0, 0.0]);
        content.x_object(Name(x.as_bytes()));
        content.restore_state();
        content.end_marked_content();
    }
    pdf.stream(content_id, &content.finish());
    Ok(pdf.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_top_level_group_is_a_layer() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" width="10" height="10"><defs><linearGradient id="g"/></defs><g id="Lienzo" data-name="Lienzo"><rect width="10" height="10"/></g><g id="cielo"><circle r="2"/></g><g><path d="M0 0"/></g><rect width="1" height="1"/></svg>"##;
        let layers = split_layers(svg).unwrap();
        let names: Vec<&str> = layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Lienzo", "cielo", "capa 3", "formas"]);
        assert!(layers[1].svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" width="10" height="10"><defs><linearGradient id="g"/></defs><g id="cielo">"#));
        assert!(layers[1].svg.ends_with("</g></svg>"));
        assert!(split_layers("<svg").is_err());
    }
}
