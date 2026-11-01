//! Una escena para un juego (`scene.go` de pxcli): sus capas de atrás
//! adelante, cada una partida en piezas que mantienen el orden de apilado
//! (las formas propias de la capa entre sus elementos, y cada `element()`
//! aparte para que el juego lo mueva y lo anime).
//!
//! Aquí se decide qué textura sale de cada pieza y con qué nombre, se
//! recorta cada una a lo que dibuja y se escribe la escena `.tscn` (un
//! `Parallax2D` por capa con un `Sprite2D` por pieza). Rasterizar y escribir
//! los archivos es cosa de quien llama.

use std::collections::HashSet;

use super::rig::{fnum, godot_name, node_name};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScenePiece {
    /// Vacío para las formas propias de la capa.
    pub name: String,
    pub element: bool,
    pub pivot: Option<[f64; 2]>,
    /// El suyo, distinto del de su capa.
    pub parallax: Option<[f64; 2]>,
    pub line: i64,
    pub markup: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneLayer {
    pub name: String,
    /// `None`: no se dio (se mueve con el mundo).
    pub parallax: Option<[f64; 2]>,
    pub line: i64,
    pub pieces: Vec<ScenePiece>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneBackground {
    /// `#rrggbb` si es un color liso.
    pub color: String,
    pub markup: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scene {
    pub width: i64,
    pub height: i64,
    pub defs: String,
    pub background: Option<SceneBackground>,
    pub layers: Vec<SceneLayer>,
}

/// Margen transparente alrededor de cada pieza, en píxeles de la textura
/// (para que un shader que la ondule no la corte).
pub const SCENE_PAD: i64 = 8;

/// `base`, o `base_2`, `base_3`… si ya está usado; queda apuntado.
pub fn unique_name(base: &str, used: &mut HashSet<String>) -> String {
    let mut n = base.to_string();
    let mut i = 2;
    while used.contains(&n) {
        n = format!("{base}_{i}");
        i += 1;
    }
    used.insert(n.clone());
    n
}

/// Un rectángulo en píxeles: `[x0, y0)` a `[x1, y1)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PixelBox {
    pub x0: i64,
    pub y0: i64,
    pub x1: i64,
    pub y1: i64,
}

impl PixelBox {
    pub fn width(&self) -> i64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> i64 {
        self.y1 - self.y0
    }

    pub fn is_empty(&self) -> bool {
        self.x1 <= self.x0 || self.y1 <= self.y0
    }
}

/// Una textura que sale de la escena: qué se dibuja y dónde acaba.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlannedPiece {
    pub layer: String,
    pub node: String,
    pub file: String,
    pub element: bool,
    pub pivot: Option<[f64; 2]>,
    pub parallax: Option<[f64; 2]>,
    pub markup: String,
    /// En píxeles de la textura, sobre el lienzo entero; `None` si no
    /// dibuja nada.
    pub bbox: Option<PixelBox>,
}

/// La exportación decidida: nombres de la escena, de las capas y de cada
/// pieza.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScenePlan {
    pub name: String,
    pub scale: f64,
    /// La escena, con un fondo que no es un color liso ya como capa.
    pub scene: Scene,
    pub layer_nodes: Vec<String>,
    /// Los nombres ya usados en la raíz.
    pub root_names: HashSet<String>,
    pub pieces: Vec<PlannedPiece>,
}

/// Decide la exportación de `scene` (`GodotSceneExport` sin E/S): escala
/// (1 si no se da), nombre («escena» si no se da), el fondo y los nombres
/// únicos y seguros de nodos y archivos.
pub fn plan_scene(scene: Option<&Scene>, name: &str, scale: f64) -> Result<ScenePlan, String> {
    let mut sc = match scene {
        Some(s) if !s.layers.is_empty() => s.clone(),
        _ => {
            return Err(
                "el dibujo no tiene capas que exportar: usa layer() para cada franja de profundidad"
                    .into(),
            );
        }
    };
    let scale = if scale <= 0.0 { 1.0 } else { scale };
    if sc.width as f64 * scale > 4096.0 || sc.height as f64 * scale > 4096.0 {
        return Err(format!(
            "a escala {} el lienzo pasa de 4096 px: baja --scale",
            fnum(scale)
        ));
    }
    let name = if name.trim().is_empty() {
        "escena".to_string()
    } else {
        godot_name(name)
    };
    // Un fondo que no es un color liso se dibuja como una capa más, detrás.
    if let Some(bg) = &sc.background
        && bg.color.is_empty()
    {
        let layer = SceneLayer {
            name: "fondo".into(),
            pieces: vec![ScenePiece {
                markup: bg.markup.clone(),
                ..ScenePiece::default()
            }],
            ..SceneLayer::default()
        };
        sc.layers.insert(0, layer);
        sc.background = None;
    }
    let mut files = HashSet::new();
    let mut root_names = HashSet::new();
    if sc.background.as_ref().is_some_and(|b| !b.color.is_empty()) {
        // El nodo del ColorRect.
        root_names.insert("fondo".to_string());
    }
    let mut layer_nodes = Vec::new();
    let mut pieces = Vec::new();
    for l in &sc.layers {
        let ln = unique_name(&godot_name(&l.name), &mut root_names);
        let mut nodes = HashSet::new();
        for p in &l.pieces {
            let base = if p.element {
                godot_name(&p.name)
            } else {
                ln.clone()
            };
            let node = unique_name(&base, &mut nodes);
            let file = unique_name(&format!("{ln}__{node}"), &mut files) + ".png";
            pieces.push(PlannedPiece {
                layer: ln.clone(),
                node,
                file,
                element: p.element,
                pivot: p.pivot,
                parallax: p.parallax,
                markup: p.markup.clone(),
                bbox: None,
            });
        }
        layer_nodes.push(ln);
    }
    Ok(ScenePlan {
        name,
        scale,
        scene: sc,
        layer_nodes,
        root_names,
        pieces,
    })
}

impl ScenePlan {
    /// El tamaño en píxeles de la textura de una pieza (el lienzo entero).
    pub fn cell(&self) -> (i64, i64) {
        (
            (self.scene.width as f64 * self.scale) as i64,
            (self.scene.height as f64 * self.scale) as i64,
        )
    }

    /// El SVG de una pieza sobre el lienzo entero, con las definiciones de
    /// la escena. Se rasteriza con el tamaño del lienzo y `cell().0` de
    /// ancho.
    pub fn piece_svg(&self, piece: &PlannedPiece) -> String {
        let (cw, ch) = (
            super::raster::num(self.scene.width as f64),
            super::raster::num(self.scene.height as f64),
        );
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {cw} {ch}"><defs>{}</defs><svg x="0" y="0" width="{cw}" height="{ch}" viewBox="0 0 {cw} {ch}" overflow="hidden">{}</svg></svg>"#,
            self.scene.defs, piece.markup
        )
    }

    /// Las piezas que no dibujan nada (`capa/nodo`).
    pub fn empty(&self) -> Vec<String> {
        self.pieces
            .iter()
            .filter(|p| p.bbox.is_none())
            .map(|p| format!("{}/{}", p.layer, p.node))
            .collect()
    }

    /// Los elementos de la escena (para el resumen de la CLI).
    pub fn element_count(&self) -> usize {
        self.scene
            .layers
            .iter()
            .flat_map(|l| &l.pieces)
            .filter(|p| p.element)
            .count()
    }
}

/// El rectángulo más pequeño de `cell` con algún píxel visible, en una
/// imagen RGBA de `width` píxeles de ancho.
pub fn alpha_bounds(rgba: &[u8], width: i64, cell: PixelBox) -> Option<PixelBox> {
    let (mut x0, mut y0, mut x1, mut y1) = (cell.x1, cell.y1, cell.x0, cell.y0);
    for y in cell.y0..cell.y1 {
        for x in cell.x0..cell.x1 {
            if rgba[((y * width + x) * 4 + 3) as usize] > 0 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    let b = PixelBox { x0, y0, x1, y1 };
    (!b.is_empty()).then_some(b)
}

/// Lo visible de una pieza con su margen, dentro de la celda.
pub fn padded(b: PixelBox, cell: PixelBox) -> PixelBox {
    PixelBox {
        x0: (b.x0 - SCENE_PAD).max(cell.x0),
        y0: (b.y0 - SCENE_PAD).max(cell.y0),
        x1: (b.x1 + SCENE_PAD).min(cell.x1),
        y1: (b.y1 + SCENE_PAD).min(cell.y1),
    }
}

/// Recorta `b` de una imagen RGBA de `width` de ancho.
pub fn crop(rgba: &[u8], width: i64, b: PixelBox) -> Vec<u8> {
    let mut out = Vec::with_capacity((b.width() * b.height() * 4) as usize);
    for y in b.y0..b.y1 {
        let start = ((y * width + b.x0) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (b.width() * 4) as usize]);
    }
    out
}

/// `#rrggbb` en componentes de 0 a 255 (`hexRGB` de pxcli).
fn hex_rgb(s: &str) -> [f64; 3] {
    let hex: String = s.trim_start_matches('#').chars().take(6).collect();
    let v = u32::from_str_radix(&hex, 16).unwrap_or(0);
    [
        f64::from((v >> 16) & 255),
        f64::from((v >> 8) & 255),
        f64::from(v & 255),
    ]
}

/// La escena `.tscn` (`godotSceneFile`): `tex_dir` es la carpeta de las
/// texturas en `res://…/` (con la barra final).
pub fn scene_tscn(plan: &ScenePlan, tex_dir: &str) -> String {
    let sc = &plan.scene;
    let scale = plan.scale;
    let mut root_names = plan.root_names.clone();
    let mut res = String::new();
    let mut nodes = String::new();
    let mut ids = 0;
    let mut tex_id: Vec<Option<String>> = Vec::with_capacity(plan.pieces.len());
    for p in &plan.pieces {
        if p.bbox.is_none() {
            tex_id.push(None);
            continue;
        }
        ids += 1;
        let id = format!("{ids}_tex");
        res.push_str(&format!(
            "[ext_resource type=\"Texture2D\" path=\"{tex_dir}{}\" id=\"{id}\"]\n",
            p.file
        ));
        tex_id.push(Some(id));
    }
    nodes.push_str(&format!(
        "[node name=\"{}\" type=\"Node2D\"]\ntexture_filter = 2\nmetadata/ancho = {}\nmetadata/alto = {}\n\n",
        node_name(&plan.name),
        sc.width,
        sc.height
    ));
    if let Some(bg) = &sc.background
        && !bg.color.is_empty()
    {
        // Un fondo liso queda quieto detrás de todo, tan ancho como la
        // pantalla; `mouse_filter` 2 para que no pare los clics.
        let [r, g, b] = hex_rgb(&bg.color).map(|c| c / 255.0);
        nodes.push_str(
            "[node name=\"fondo\" type=\"Parallax2D\" parent=\".\"]\nscroll_scale = Vector2(0, 0)\n\n",
        );
        nodes.push_str(&format!(
            "[node name=\"color\" type=\"ColorRect\" parent=\"fondo\"]\noffset_right = {}\noffset_bottom = {}\nmouse_filter = 2\ncolor = Color({}, {}, {}, 1)\n\n",
            sc.width,
            sc.height,
            fnum(r),
            fnum(g),
            fnum(b)
        ));
    }
    let mut k = 0;
    for (li, l) in sc.layers.iter().enumerate() {
        let p = l.parallax.unwrap_or([1.0, 1.0]);
        // Un Parallax2D por capa; un elemento con parallax propio tiene el
        // suyo, en su sitio del apilado, y el resto de la capa sigue en otro.
        let mut parent = String::new();
        let mut open = [0.0, 0.0];
        let group = |nodes: &mut String,
                     node: String,
                     scroll: [f64; 2],
                     parent: &mut String,
                     open: &mut [f64; 2]| {
            nodes.push_str(&format!(
                "[node name=\"{node}\" type=\"Parallax2D\" parent=\".\"]\nscroll_scale = Vector2({}, {})\nmetadata/capa = {}\n\n",
                fnum(scroll[0]),
                fnum(scroll[1]),
                crate::error::go_quote(&l.name)
            ));
            *parent = node;
            *open = scroll;
        };
        group(
            &mut nodes,
            plan.layer_nodes[li].clone(),
            p,
            &mut parent,
            &mut open,
        );
        for _ in &l.pieces {
            let pc = &plan.pieces[k];
            let id = tex_id[k].clone();
            k += 1;
            let (Some(id), Some(bbox)) = (id, pc.bbox) else {
                continue;
            };
            let want = pc.parallax.unwrap_or(p);
            if want != open {
                if want == p {
                    let node = unique_name(&plan.layer_nodes[li], &mut root_names);
                    group(&mut nodes, node, p, &mut parent, &mut open);
                } else {
                    let node = unique_name(&pc.node, &mut root_names);
                    group(&mut nodes, node, want, &mut parent, &mut open);
                }
            }
            let (x, y) = (bbox.x0 as f64 / scale, bbox.y0 as f64 / scale);
            let mut text = format!(
                "[node name=\"{}\" type=\"Sprite2D\" parent=\"{parent}\"]\ntexture = ExtResource(\"{id}\")\ncentered = false\n",
                pc.node
            );
            if scale != 1.0 {
                text.push_str(&format!(
                    "scale = Vector2({}, {})\n",
                    fnum(1.0 / scale),
                    fnum(1.0 / scale)
                ));
            }
            if pc.element {
                // El nodo está en el pivote y la textura se desplaza para
                // dibujarse en su sitio.
                let (px, py) = match pc.pivot {
                    Some(pv) => (pv[0], pv[1]),
                    None => (
                        x + bbox.width() as f64 / scale / 2.0,
                        y + bbox.height() as f64 / scale / 2.0,
                    ),
                };
                text.push_str(&format!(
                    "position = Vector2({}, {})\noffset = Vector2({}, {})\nmetadata/elemento = true\n\n",
                    fnum(px),
                    fnum(py),
                    fnum((x - px) * scale),
                    fnum((y - py) * scale)
                ));
            } else {
                text.push_str(&format!("position = Vector2({}, {})\n\n", fnum(x), fnum(y)));
            }
            nodes.push_str(&text);
        }
    }
    format!(
        "[gd_scene load_steps={} format=3]\n\n{res}\n{nodes}",
        ids + 1
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_safe() {
        let mut used = HashSet::new();
        assert_eq!(unique_name("cielo", &mut used), "cielo");
        assert_eq!(unique_name("cielo", &mut used), "cielo_2");
        assert_eq!(unique_name("cielo", &mut used), "cielo_3");
        assert_eq!(godot_name("  Árbol Grande! "), "arbol_grande");
        assert_eq!(godot_name("¿?"), "personaje");
    }

    #[test]
    fn the_visible_box_has_its_margin_inside_the_cell() {
        let width = 20;
        let mut rgba = vec![0u8; 20 * 10 * 4];
        rgba[((3 * width + 12) * 4 + 3) as usize] = 255;
        rgba[((5 * width + 15) * 4 + 3) as usize] = 1;
        let cell = PixelBox {
            x0: 0,
            y0: 0,
            x1: 20,
            y1: 10,
        };
        let b = alpha_bounds(&rgba, width, cell).unwrap();
        assert_eq!(
            b,
            PixelBox {
                x0: 12,
                y0: 3,
                x1: 16,
                y1: 6
            }
        );
        assert_eq!(
            padded(b, cell),
            PixelBox {
                x0: 4,
                y0: 0,
                x1: 20,
                y1: 10
            }
        );
        assert_eq!(
            alpha_bounds(
                &[0; 16],
                2,
                PixelBox {
                    x0: 0,
                    y0: 0,
                    x1: 2,
                    y1: 2
                }
            ),
            None
        );
        assert_eq!(crop(&rgba, width, b).len(), (4 * 3 * 4) as usize);
    }
}
