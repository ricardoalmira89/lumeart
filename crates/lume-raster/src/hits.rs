//! Qué formas de un dibujo vectorial se ven en una zona o en un punto (el
//! lazo y la herramienta de selección del modo vector).
//!
//! La página de pxcli muestreaba unos 500 puntos dentro de la zona y
//! preguntaba al navegador qué formas había bajo cada uno. Aquí cada forma
//! candidata (la que su caja toca la zona) se pinta sola con resvg sobre una
//! rejilla de esos puntos, y se cuenta en cuántos se ve y en cuántos es la de
//! encima. No se aplican los recortes ni las máscaras de los grupos que la
//! rodean: una forma recortada cuenta un poco más de lo que se ve.

use std::collections::HashMap;

use lume_app::ports::{PortError, ShapeFinder};
use lume_domain::vec::studio::{ShapeHit, bounds, in_polygon};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, roxmltree};

use crate::ResvgRasterizer;

/// Prefijo del `id` con que se encuentra cada forma en el árbol de usvg.
const ID_PREFIX: &str = "lumehit-";
/// Puntos que se miran dentro de una zona (más o menos).
const SAMPLES: f64 = 500.0;
/// Un punto está cubierto por una forma si su alfa pasa de esta fracción
/// del alfa máximo de la forma: así cuentan también las semitransparentes
/// (un halo con opacidad 0,18), como en el navegador, que no mira la
/// opacidad al buscar qué hay bajo un punto.
const COVERED: f32 = 0.25;
/// Una forma que ocupa más de esto del lienzo es «grande» (fondo, luz,
/// grano): cuenta, pero nunca como la de encima.
const BIG: f64 = 0.6;

/// Una forma del SVG del studio.
#[derive(Debug, Clone)]
struct Leaf {
    index: usize,
    id: String,
    line: u32,
    chain: Vec<u32>,
    tag: String,
    layer: String,
}

/// Las formas (con `data-i`) y el SVG con un `id` en cada una.
fn leaves(studio_svg: &str) -> Result<(Vec<Leaf>, String), PortError> {
    let doc = roxmltree::Document::parse(studio_svg)
        .map_err(|e| PortError::Invalid(format!("SVG no válido: {e}")))?;
    let mut out = Vec::new();
    for node in doc.descendants().filter(|n| n.is_element()) {
        let Some(index) = node.attribute("data-i").and_then(|i| i.parse().ok()) else {
            continue;
        };
        let line = node
            .attribute("data-l")
            .and_then(|l| l.parse().ok())
            .unwrap_or(0);
        let chain = node
            .attribute("data-c")
            .map(|c| c.split(',').filter_map(|l| l.trim().parse().ok()).collect())
            .unwrap_or_default();
        let layer = node
            .ancestors()
            .skip(1)
            .filter(|a| a.is_element() && a.parent().is_some_and(|p| p.is_element()))
            .filter_map(|a| a.attribute("data-name"))
            .last()
            .unwrap_or_default()
            .to_string();
        let id = node
            .attribute("id")
            .map(str::to_string)
            .unwrap_or_else(|| format!("{ID_PREFIX}{index}"));
        out.push(Leaf {
            index,
            id,
            line,
            chain,
            tag: node.tag_name().name().to_string(),
            layer,
        });
    }
    // Un `id` en cada forma que no tenga.
    const KEY: &str = " data-i=\"";
    let mut tagged = String::with_capacity(studio_svg.len() + out.len() * 16);
    let mut from = 0;
    while let Some(i) = studio_svg[from..].find(KEY) {
        let at = from + i;
        let value_end = studio_svg[at + KEY.len()..]
            .find('"')
            .map(|e| at + KEY.len() + e + 1)
            .unwrap_or(studio_svg.len());
        tagged.push_str(&studio_svg[from..value_end]);
        let tag_start = studio_svg[..at].rfind('<').unwrap_or(0);
        let tag_end = studio_svg[at..]
            .find('>')
            .map_or(studio_svg.len(), |e| at + e);
        if !studio_svg[tag_start..tag_end].contains(" id=\"") {
            let index = &studio_svg[at + KEY.len()..value_end - 1];
            tagged.push_str(&format!(" id=\"{ID_PREFIX}{index}\""));
        }
        from = value_end;
    }
    tagged.push_str(&studio_svg[from..]);
    Ok((out, tagged))
}

/// Lo que cubre cada forma en una rejilla de `cols` × `rows` puntos que
/// empieza en `origin` con paso `step` (en unidades del dibujo).
struct Grid {
    origin: [f64; 2],
    step: f64,
    cols: u32,
    rows: u32,
}

impl Grid {
    /// Los puntos de la rejilla cubiertos por `node` (por índice).
    fn coverage(&self, node: &usvg::Node) -> Option<Vec<bool>> {
        let mut pixmap = Pixmap::new(self.cols, self.rows)?;
        let to_grid = Transform::from_row(
            (1.0 / self.step) as f32,
            0.0,
            0.0,
            (1.0 / self.step) as f32,
            (-self.origin[0] / self.step) as f32,
            (-self.origin[1] / self.step) as f32,
        );
        // resvg pinta el nodo con su propia transformación y desplazado a su
        // caja: se le da la de su padre y se compensa el desplazamiento.
        let parent = match node {
            usvg::Node::Group(g) => g
                .abs_transform()
                .pre_concat(g.transform().invert().unwrap_or_default()),
            other => other.abs_transform(),
        };
        let bbox = node.abs_layer_bounding_box()?;
        let transform = to_grid.pre_concat(parent).pre_translate(bbox.x(), bbox.y());
        resvg::render_node(node, transform, &mut pixmap.as_mut())?;
        let max = pixmap.pixels().iter().map(|p| p.alpha()).max().unwrap_or(0);
        let limit = (f32::from(max) * COVERED).max(1.0);
        Some(
            pixmap
                .pixels()
                .iter()
                .map(|p| f32::from(p.alpha()) > limit)
                .collect(),
        )
    }
}

fn parse(svg: &str) -> Result<usvg::Tree, PortError> {
    usvg::Tree::from_str(svg, &crate::options())
        .map_err(|e| PortError::Invalid(format!("SVG no válido: {e}")))
}

/// Lo que ve cada punto de `samples` (índices de la rejilla): por forma
/// encontrada, en cuántos se ve y en cuántos está encima.
fn count(
    tree: &usvg::Tree,
    leaves: &[Leaf],
    grid: &Grid,
    samples: &[usize],
    canvas_area: f64,
    area: [f64; 4],
) -> Vec<(usize, usize, usize, bool)> {
    let mut covered: Vec<(usize, Vec<bool>, bool)> = Vec::new();
    for (k, leaf) in leaves.iter().enumerate() {
        let Some(node) = tree.node_by_id(&leaf.id) else {
            continue;
        };
        let b = node.abs_stroke_bounding_box();
        if b.right() < area[0] as f32
            || b.left() > area[2] as f32
            || b.bottom() < area[1] as f32
            || b.top() > area[3] as f32
        {
            continue;
        }
        let big = f64::from(b.width()) * f64::from(b.height()) > canvas_area * BIG;
        if let Some(cells) = grid.coverage(node) {
            covered.push((k, cells, big));
        }
    }
    // De arriba abajo en el orden de dibujo.
    covered.sort_by(|a, b| leaves[b.0].index.cmp(&leaves[a.0].index));
    let mut out: Vec<(usize, usize, usize, bool)> =
        covered.iter().map(|(k, _, big)| (*k, 0, 0, *big)).collect();
    for &s in samples {
        let mut first = true;
        for (entry, (_, cells, big)) in out.iter_mut().zip(&covered) {
            if cells[s] {
                entry.1 += 1;
                if first && !big {
                    entry.2 += 1;
                    first = false;
                }
            }
        }
    }
    out.retain(|e| e.1 > 0);
    out
}

impl ShapeFinder for ResvgRasterizer {
    fn shapes_under(
        &self,
        studio_svg: &str,
        width: f64,
        height: f64,
        zone: &[[f64; 2]],
    ) -> Result<Vec<ShapeHit>, PortError> {
        if zone.len() < 3 {
            return Ok(Vec::new());
        }
        let (leaves, tagged) = leaves(studio_svg)?;
        let tree = parse(&tagged)?;
        let area = bounds(zone);
        let (bw, bh) = ((area[2] - area[0]).max(1e-6), (area[3] - area[1]).max(1e-6));
        let step = (bw * bh / SAMPLES).sqrt().max(0.25);
        let grid = Grid {
            origin: [area[0], area[1]],
            step,
            cols: (bw / step).ceil().max(1.0) as u32,
            rows: (bh / step).ceil().max(1.0) as u32,
        };
        let samples: Vec<usize> = (0..grid.rows)
            .flat_map(|j| (0..grid.cols).map(move |i| (i, j)))
            .filter(|&(i, j)| {
                let x = area[0] + (f64::from(i) + 0.5) * step;
                let y = area[1] + (f64::from(j) + 0.5) * step;
                in_polygon(x, y, zone)
            })
            .map(|(i, j)| (j * grid.cols + i) as usize)
            .collect();
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        let inside = samples.len() as f64;
        let counts = count(&tree, &leaves, &grid, &samples, width * height, area);
        // Una entrada por llamada (las formas de un mismo bucle o función).
        let mut groups: Vec<ShapeHit> = Vec::new();
        let mut by_key: HashMap<String, usize> = HashMap::new();
        for (k, n, top, big) in counts {
            let leaf = &leaves[k];
            if leaf.line == 0 {
                continue;
            }
            let key = if leaf.chain.is_empty() {
                leaf.line.to_string()
            } else {
                format!("{:?}", leaf.chain)
            };
            let i = *by_key.entry(key).or_insert_with(|| {
                groups.push(ShapeHit {
                    line: leaf.line,
                    chain: leaf.chain.clone(),
                    tag: if leaf.tag == "g" {
                        "objeto de VectorCraft".into()
                    } else {
                        leaf.tag.clone()
                    },
                    layer: leaf.layer.clone(),
                    share: 0.0,
                    top: 0.0,
                    big,
                });
                groups.len() - 1
            });
            groups[i].share = (groups[i].share + n as f64 / inside).min(1.0);
            groups[i].top = (groups[i].top + top as f64 / inside).min(1.0);
        }
        groups.sort_by(|a, b| b.share.total_cmp(&a.share));
        groups.truncate(120);
        for g in &mut groups {
            g.share = (g.share * 1000.0).round() / 1000.0;
            g.top = (g.top * 1000.0).round() / 1000.0;
        }
        Ok(groups)
    }

    fn shape_at(
        &self,
        studio_svg: &str,
        width: f64,
        height: f64,
        point: [f64; 2],
    ) -> Result<Option<ShapeHit>, PortError> {
        let (leaves, tagged) = leaves(studio_svg)?;
        let tree = parse(&tagged)?;
        let grid = Grid {
            origin: [point[0] - 0.5, point[1] - 0.5],
            step: 1.0,
            cols: 1,
            rows: 1,
        };
        let area = [point[0], point[1], point[0], point[1]];
        let counts = count(&tree, &leaves, &grid, &[0], width * height, area);
        // La de encima que no cubre todo; si solo hay de esas, la de encima.
        let pick = counts
            .iter()
            .find(|c| !c.3)
            .or_else(|| counts.first())
            .map(|c| &leaves[c.0]);
        Ok(pick.filter(|l| l.line > 0).map(|leaf| ShapeHit {
            line: leaf.line,
            chain: leaf.chain.clone(),
            tag: leaf.tag.clone(),
            layer: leaf.layer.clone(),
            share: 1.0,
            top: 1.0,
            big: false,
        }))
    }
}
