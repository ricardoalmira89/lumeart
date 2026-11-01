//! Un personaje con esqueleto (`rig.go` y `rigpose.go` de pxcli): huesos
//! con sus articulaciones, las piezas dibujadas en ellos (en orden de
//! apilado) y las acciones, tal como los declara `dibujo.js`. Las
//! coordenadas son las del dibujo en la pose de reposo.
//!
//! Aquí se calcula todo lo que no es E/S: el atlas de piezas (dónde va cada
//! una y el SVG que se rasteriza), la escena `.tscn` de Godot 4 (`Node2D` >
//! `Skeleton2D` > `Bone2D`… con un `Sprite2D` por pieza y un
//! `AnimationPlayer` con cada acción) y el SVG de `vec poses`, con la misma
//! interpolación que Godot. Las cuentas usan el `math` y el `sort.Slice` de
//! Go para dar los mismos números y el mismo orden que pxcli.

use std::collections::BTreeMap;
use std::f64::consts::PI;

use super::raster::num;
use crate::gomath::{atan2, cos, go_hypot, sin};
use crate::gosort::sort_slice;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RigBone {
    pub name: String,
    pub parent: String,
    pub at: [f64; 2],
    pub line: i64,
}

/// Una pose de un hueso: grados (en el sentido de las agujas en pantalla),
/// desplazamiento en el espacio del padre, escala alrededor de la
/// articulación y opacidad (las dos heredan a los hijos, como en Godot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigPose {
    pub rot: f64,
    pub x: f64,
    pub y: f64,
    pub sx: f64,
    pub sy: f64,
    pub alpha: f64,
}

impl Default for RigPose {
    fn default() -> Self {
        RigPose {
            rot: 0.0,
            x: 0.0,
            y: 0.0,
            sx: 1.0,
            sy: 1.0,
            alpha: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RigKey {
    /// De 0 a 1 a lo largo de la acción.
    pub t: f64,
    pub pose: BTreeMap<String, RigPose>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RigAction {
    pub name: String,
    pub duration: f64,
    pub looping: bool,
    /// Unidades por segundo que avanza el personaje mientras la hace.
    pub movement: f64,
    pub keys: Vec<RigKey>,
    pub line: i64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RigPart {
    pub name: String,
    pub bone: String,
    pub z: i64,
    /// `x, y, ancho, alto` en unidades del dibujo.
    pub bbox: [f64; 4],
    pub markup: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rig {
    pub bones: Vec<RigBone>,
    pub actions: Vec<RigAction>,
    pub origin: [f64; 2],
    pub width: i64,
    pub height: i64,
    pub defs: String,
    pub parts: Vec<RigPart>,
}

/// `fnum` de pxcli: cinco decimales como mucho, sin ceros al final, y 0
/// para lo que es casi 0.
pub fn fnum(v: f64) -> String {
    if v.abs() < 1e-9 {
        return "0".into();
    }
    let s = format!("{v:.5}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Un nombre válido para Godot: minúsculas, sin acentos, palabras unidas
/// con `_`. Vacío: «personaje».
pub fn godot_name(s: &str) -> String {
    let lower = s.trim().to_lowercase();
    let plain: String = lower
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect();
    let mut out = String::new();
    for c in plain.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }
    let n = out.trim_matches('_');
    if n.is_empty() {
        "personaje".into()
    } else {
        n.to_string()
    }
}

/// El nombre con la primera letra en mayúscula (el del nodo raíz).
pub fn node_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// La escala de la textura que mantiene nítido al personaje aunque el
/// juego lo muestre tan alto como una pantalla de 1080p: unos 1200 píxeles
/// de la cabeza a los pies, entre 1 y 4 por unidad.
pub fn auto_scale(rig: &Rig) -> f64 {
    if rig.parts.is_empty() {
        return 2.0;
    }
    let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
    for p in &rig.parts {
        y0 = y0.min(p.bbox[1]);
        y1 = y1.max(p.bbox[1] + p.bbox[3]);
    }
    let k = (1200.0 / (y1 - y0).max(1.0) * 4.0).round() / 4.0;
    k.clamp(1.0, 4.0)
}

/// La celda de una pieza en el atlas, en píxeles de la textura.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AtlasCell {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

/// Coloca las piezas en estantes (de la más alta a la más baja), cada una
/// en una celda del tamaño de su caja.
pub fn pack_atlas(parts: &[RigPart], scale: f64) -> Vec<AtlasCell> {
    const GAP: i64 = 2;
    const MAX_W: i64 = 2048;
    let mut cells: Vec<AtlasCell> = parts
        .iter()
        .map(|p| AtlasCell {
            x: 0,
            y: 0,
            w: ((p.bbox[2] * scale).ceil() as i64).max(1),
            h: ((p.bbox[3] * scale).ceil() as i64).max(1),
        })
        .collect();
    let mut order: Vec<usize> = (0..parts.len()).collect();
    sort_slice(&mut order, |a, b| cells[*a].h > cells[*b].h);
    let (mut x, mut y, mut row) = (GAP, GAP, 0);
    for i in order {
        if x + cells[i].w + GAP > MAX_W && x > GAP {
            x = GAP;
            y += row + GAP;
            row = 0;
        }
        cells[i].x = x;
        cells[i].y = y;
        x += cells[i].w + GAP;
        row = row.max(cells[i].h);
    }
    cells
}

/// El atlas que hay que rasterizar: el SVG, su tamaño en unidades y el
/// ancho de salida (`Rasterize(svg, ceil(aw/escala), ceil(ah/escala), aw)`).
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasSvg {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub out_width: u32,
}

/// El SVG del atlas: cada pieza en su celda.
pub fn atlas_svg(rig: &Rig, cells: &[AtlasCell], scale: f64) -> AtlasSvg {
    let aw = cells.iter().map(|c| c.x + c.w).max().unwrap_or(0);
    let ah = cells.iter().map(|c| c.y + c.h).max().unwrap_or(0);
    let mut b = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{aw}" height="{ah}"><defs>{}</defs>"#,
        num(aw as f64 / scale),
        num(ah as f64 / scale),
        rig.defs
    );
    for (p, c) in rig.parts.iter().zip(cells) {
        b.push_str(&format!(
            r#"<svg x="{}" y="{}" width="{}" height="{}" viewBox="{} {} {} {}" preserveAspectRatio="none">{}</svg>"#,
            num(c.x as f64 / scale),
            num(c.y as f64 / scale),
            num(c.w as f64 / scale),
            num(c.h as f64 / scale),
            num(p.bbox[0]),
            num(p.bbox[1]),
            num(c.w as f64 / scale),
            num(c.h as f64 / scale),
            p.markup
        ));
    }
    b.push_str("</svg>");
    AtlasSvg {
        svg: b,
        width: (aw as f64 / scale).ceil(),
        height: (ah as f64 / scale).ceil(),
        out_width: aw.max(0) as u32,
    }
}

/// Las claves de un hueso en una acción, como las reproducirá Godot.
#[derive(Debug, Clone, Default)]
struct BoneTrack {
    times: Vec<f64>,
    /// Radianes.
    rot: Vec<f64>,
    pos: Vec<[f64; 2]>,
    scale: Vec<[f64; 2]>,
    alpha: Vec<f64>,
    rot_used: bool,
    pos_used: bool,
    scale_used: bool,
    alpha_used: bool,
    rest_pos: [f64; 2],
}

impl Rig {
    fn bone(&self, name: &str) -> Option<&RigBone> {
        self.bones.iter().find(|b| b.name == name)
    }

    fn bone_index(&self, name: &str) -> usize {
        self.bones
            .iter()
            .position(|b| b.name == name)
            .unwrap_or(self.bones.len())
    }

    /// La ruta del nodo de un hueso desde la raíz del personaje.
    fn bone_path(&self, name: &str) -> String {
        let mut chain: Vec<&str> = Vec::new();
        let mut b = self.bone(name);
        while let Some(bone) = b {
            chain.insert(0, &bone.name);
            if bone.parent.is_empty() {
                break;
            }
            b = self.bone(&bone.parent);
        }
        format!("Skeleton2D/{}", chain.join("/"))
    }

    /// Por cada hueso que se mueve en la acción, sus claves: un hueso que
    /// una clave no menciona no tiene clave ahí (Godot interpola), reposa
    /// en 0 si la primera clave no lo menciona y un bucle acaba donde
    /// empieza.
    fn tracks(
        &self,
        a: &RigAction,
        rest: &BTreeMap<String, [f64; 2]>,
    ) -> (BTreeMap<String, BoneTrack>, Vec<String>) {
        let mut out: BTreeMap<String, BoneTrack> = BTreeMap::new();
        let mut order: Vec<String> = Vec::new();
        for k in &a.keys {
            for bn in k.pose.keys() {
                if !out.contains_key(bn) {
                    out.insert(
                        bn.clone(),
                        BoneTrack {
                            rest_pos: rest.get(bn).copied().unwrap_or_default(),
                            ..BoneTrack::default()
                        },
                    );
                    order.push(bn.clone());
                }
            }
        }
        sort_slice(&mut order, |i, j| self.bone_index(i) < self.bone_index(j));
        for bn in &order {
            let t = out.get_mut(bn).expect("pista");
            for k in &a.keys {
                let Some(p) = k.pose.get(bn) else { continue };
                t.times.push(k.t * a.duration);
                t.rot.push(p.rot * PI / 180.0);
                t.pos.push([t.rest_pos[0] + p.x, t.rest_pos[1] + p.y]);
                t.scale.push([p.sx, p.sy]);
                t.alpha.push(p.alpha);
                if p.sx != 1.0 || p.sy != 1.0 {
                    t.scale_used = true;
                }
                if p.alpha != 1.0 {
                    t.alpha_used = true;
                }
                if p.rot != 0.0 {
                    t.rot_used = true;
                }
                if p.x != 0.0 || p.y != 0.0 {
                    t.pos_used = true;
                }
            }
            if t.times.is_empty() || t.times[0] > 1e-6 {
                t.times.insert(0, 0.0);
                t.rot.insert(0, 0.0);
                t.pos.insert(0, t.rest_pos);
                t.scale.insert(0, [1.0, 1.0]);
                t.alpha.insert(0, 1.0);
            }
            if a.looping && *t.times.last().expect("clave") < a.duration - 1e-6 {
                t.times.push(a.duration);
                t.rot.push(t.rot[0]);
                t.pos.push(t.pos[0]);
                t.scale.push(t.scale[0]);
                t.alpha.push(t.alpha[0]);
            }
        }
        self.unwind(&mut out);
        (out, order)
    }

    /// Quita vueltas enteras a cada pista, para que Godot no dé una vuelta
    /// completa al pasar de una acción a otra: cada pista se queda con las
    /// vueltas que dejan su primera clave más cerca del reposo, contando su
    /// giro y su giro en pantalla (con los de los huesos de arriba).
    fn unwind(&self, out: &mut BTreeMap<String, BoneTrack>) {
        let depth = |name: &str| {
            let mut d = 0;
            let mut b = self.bone(name);
            while let Some(bone) = b {
                if bone.parent.is_empty() {
                    break;
                }
                d += 1;
                b = self.bone(&bone.parent);
            }
            d
        };
        let mut names: Vec<String> = out.keys().cloned().collect();
        sort_slice(&mut names, |i, j| {
            let (di, dj) = (depth(i), depth(j));
            if di != dj { di < dj } else { i < j }
        });
        for bn in names {
            let mut above = 0.0;
            let mut b = self.bone(&bn);
            while let Some(bone) = b {
                if bone.parent.is_empty() {
                    break;
                }
                if let Some(pt) = out.get(&bone.parent) {
                    above += pt.rot[0];
                }
                b = self.bone(&bone.parent);
            }
            let t = out.get_mut(&bn).expect("pista");
            let near = (t.rot[0] / (2.0 * PI)).round();
            let (mut best, mut best_cost) = (near, f64::INFINITY);
            for k in [near, near - 1.0, near + 1.0, near - 2.0, near + 2.0] {
                let own = t.rot[0] - k * 2.0 * PI;
                let cost = own.abs().max((above + own).abs());
                if cost < best_cost - 1e-9 {
                    best = k;
                    best_cost = cost;
                }
            }
            if best != 0.0 {
                for r in &mut t.rot {
                    *r -= best * 2.0 * PI;
                }
            }
        }
    }

    /// Lo que un juego necesita para colocar y mover al personaje, como
    /// metadatos de su nodo raíz: `alto` (sobre el origen) y `velocidad` de
    /// cada acción que avanza.
    fn godot_meta(&self) -> String {
        let mut top = self.origin[1];
        for p in &self.parts {
            top = top.min(p.bbox[1]);
        }
        let speeds: Vec<String> = self
            .actions
            .iter()
            .filter(|a| a.movement != 0.0)
            .map(|a| format!("{}: {}", crate::error::go_quote(&a.name), fnum(a.movement)))
            .collect();
        format!(
            "metadata/alto = {}\nmetadata/velocidad = {{{}}}\n",
            fnum(self.origin[1] - top),
            speeds.join(", ")
        )
    }

    /// La posición de reposo de cada hueso respecto a su padre (o al
    /// origen).
    fn rest_positions(&self) -> BTreeMap<String, [f64; 2]> {
        self.bones
            .iter()
            .map(|b| {
                let base = match self.bone(&b.parent) {
                    Some(p) => p.at,
                    None => self.origin,
                };
                (b.name.clone(), [b.at[0] - base[0], b.at[1] - base[1]])
            })
            .collect()
    }
}

struct TrackOut {
    path: String,
    prop: &'static str,
    values: String,
    times: Vec<f64>,
    interp: u8,
}

fn write_anim(
    res: &mut String,
    id: &str,
    name: &str,
    length: f64,
    looping: bool,
    tracks: &[TrackOut],
) {
    res.push_str(&format!(
        "[sub_resource type=\"Animation\" id=\"{id}\"]\nresource_name = \"{name}\"\nlength = {}\n",
        fnum(length)
    ));
    if looping {
        res.push_str("loop_mode = 1\n");
    }
    for (i, t) in tracks.iter().enumerate() {
        let times: Vec<String> = t.times.iter().map(|v| fnum(*v)).collect();
        let trans: Vec<&str> = t.times.iter().map(|_| "1").collect();
        res.push_str(&format!(
            "tracks/{i}/type = \"value\"\ntracks/{i}/imported = false\ntracks/{i}/enabled = true\ntracks/{i}/path = NodePath(\"{}:{}\")\ntracks/{i}/interp = {}\ntracks/{i}/loop_wrap = true\ntracks/{i}/keys = {{\n\"times\": PackedFloat32Array({}),\n\"transitions\": PackedFloat32Array({}),\n\"update\": 0,\n\"values\": [{}]\n}}\n",
            t.path,
            t.prop,
            t.interp,
            times.join(", "),
            trans.join(", "),
            t.values
        ));
    }
    res.push('\n');
}

/// La escena `.tscn` del personaje, con la textura del atlas en `tex_path`
/// (`res://…`).
pub fn godot_scene(
    rig: &Rig,
    name: &str,
    tex_path: &str,
    cells: &[AtlasCell],
    scale: f64,
) -> String {
    let rest = rig.rest_positions();
    let mut res = String::new();
    let mut nodes = String::new();
    let mut subs = 0;
    res.push_str(&format!(
        "[ext_resource type=\"Texture2D\" path=\"{tex_path}\" id=\"1_piezas\"]\n\n"
    ));
    for (p, c) in rig.parts.iter().zip(cells) {
        res.push_str(&format!(
            "[sub_resource type=\"AtlasTexture\" id=\"AtlasTexture_{}\"]\natlas = ExtResource(\"1_piezas\")\nregion = Rect2({}, {}, {}, {})\n\n",
            p.name, c.x, c.y, c.w, c.h
        ));
        subs += 1;
    }
    // RESET (la pose de reposo, para el editor) y una animación por acción.
    // La escala y la opacidad solo se reponen en los huesos que las usan.
    let (mut scaled, mut faded) = (Vec::new(), Vec::new());
    for a in &rig.actions {
        for k in &a.keys {
            for (bn, p) in &k.pose {
                if p.sx != 1.0 || p.sy != 1.0 {
                    scaled.push(bn.clone());
                }
                if p.alpha != 1.0 {
                    faded.push(bn.clone());
                }
            }
        }
    }
    let mut reset = Vec::new();
    for b in &rig.bones {
        let rp = rest[&b.name];
        reset.push(TrackOut {
            path: rig.bone_path(&b.name),
            prop: "rotation",
            values: "0.0".into(),
            times: vec![0.0],
            interp: 1,
        });
        reset.push(TrackOut {
            path: rig.bone_path(&b.name),
            prop: "position",
            values: format!("Vector2({}, {})", fnum(rp[0]), fnum(rp[1])),
            times: vec![0.0],
            interp: 1,
        });
        if scaled.contains(&b.name) {
            reset.push(TrackOut {
                path: rig.bone_path(&b.name),
                prop: "scale",
                values: "Vector2(1, 1)".into(),
                times: vec![0.0],
                interp: 1,
            });
        }
        if faded.contains(&b.name) {
            reset.push(TrackOut {
                path: rig.bone_path(&b.name),
                prop: "modulate",
                values: "Color(1, 1, 1, 1)".into(),
                times: vec![0.0],
                interp: 1,
            });
        }
    }
    write_anim(&mut res, "Animation_RESET", "RESET", 0.001, false, &reset);
    subs += 1;
    let mut lib = Vec::new();
    for a in &rig.actions {
        let (tracks, order) = rig.tracks(a, &rest);
        let mut out = Vec::new();
        for bn in &order {
            let t = &tracks[bn];
            if t.rot_used {
                let vals: Vec<String> = t.rot.iter().map(|v| fnum(*v)).collect();
                out.push(TrackOut {
                    path: rig.bone_path(bn),
                    prop: "rotation",
                    values: vals.join(", "),
                    times: t.times.clone(),
                    interp: 2,
                });
            }
            if t.pos_used {
                let vals: Vec<String> = t
                    .pos
                    .iter()
                    .map(|v| format!("Vector2({}, {})", fnum(v[0]), fnum(v[1])))
                    .collect();
                out.push(TrackOut {
                    path: rig.bone_path(bn),
                    prop: "position",
                    values: vals.join(", "),
                    times: t.times.clone(),
                    interp: 2,
                });
            }
            if t.scale_used {
                let vals: Vec<String> = t
                    .scale
                    .iter()
                    .map(|v| format!("Vector2({}, {})", fnum(v[0]), fnum(v[1])))
                    .collect();
                out.push(TrackOut {
                    path: rig.bone_path(bn),
                    prop: "scale",
                    values: vals.join(", "),
                    times: t.times.clone(),
                    interp: 2,
                });
            }
            if t.alpha_used {
                let vals: Vec<String> = t
                    .alpha
                    .iter()
                    .map(|v| format!("Color(1, 1, 1, {})", fnum(*v)))
                    .collect();
                out.push(TrackOut {
                    path: rig.bone_path(bn),
                    prop: "modulate",
                    values: vals.join(", "),
                    times: t.times.clone(),
                    interp: 2,
                });
            }
        }
        write_anim(
            &mut res,
            &format!("Animation_{}", a.name),
            &a.name,
            a.duration,
            a.looping,
            &out,
        );
        subs += 1;
        lib.push(format!("&\"{0}\": SubResource(\"Animation_{0}\")", a.name));
    }
    lib.push("&\"RESET\": SubResource(\"Animation_RESET\")".into());
    res.push_str(&format!(
        "[sub_resource type=\"AnimationLibrary\" id=\"AnimationLibrary_{name}\"]\n_data = {{\n{}\n}}\n\n",
        lib.join(",\n")
    ));
    subs += 1;

    // Nodos: el origen del personaje es el punto del suelo; las piezas se
    // apilan por z.
    nodes.push_str(&format!(
        "[node name=\"{}\" type=\"Node2D\"]\ntexture_filter = 2\n{}\n[node name=\"Skeleton2D\" type=\"Skeleton2D\" parent=\".\"]\n\n",
        node_name(name),
        rig.godot_meta()
    ));
    for b in &rig.bones {
        let rp = rest[&b.name];
        let (mut length, mut angle) = (16.0, 0.0);
        if let Some(c) = rig.bones.iter().find(|c| c.parent == b.name) {
            let (dx, dy) = (c.at[0] - b.at[0], c.at[1] - b.at[1]);
            length = go_hypot(dx, dy).max(8.0);
            angle = atan2(dy, dx) * 180.0 / PI;
        }
        // Un hueso sin hijos apunta hacia su primera pieza.
        if length == 16.0
            && let Some(p) = rig.parts.iter().find(|p| p.bone == b.name)
        {
            let dx = p.bbox[0] + p.bbox[2] / 2.0 - b.at[0];
            let dy = p.bbox[1] + p.bbox[3] / 2.0 - b.at[1];
            length = (go_hypot(dx, dy) * 2.0).max(8.0);
            angle = atan2(dy, dx) * 180.0 / PI;
        }
        let path = rig.bone_path(&b.name);
        let parent_path = path.strip_suffix(&format!("/{}", b.name)).unwrap_or(&path);
        nodes.push_str(&format!(
            "[node name=\"{}\" type=\"Bone2D\" parent=\"{parent_path}\"]\nposition = Vector2({}, {})\nrest = Transform2D(1, 0, 0, 1, {}, {})\nauto_calculate_length_and_angle = false\nlength = {}\nbone_angle = {}\n\n",
            b.name,
            fnum(rp[0]),
            fnum(rp[1]),
            fnum(rp[0]),
            fnum(rp[1]),
            fnum(length),
            fnum(angle)
        ));
    }
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &rig.parts {
        let at = rig.bone(&p.bone).map_or([0.0, 0.0], |b| b.at);
        let n = count.entry(&p.bone).or_default();
        *n += 1;
        let node = if *n > 1 {
            format!("pieza{n}")
        } else {
            "pieza".into()
        };
        nodes.push_str(&format!(
            "[node name=\"{node}\" type=\"Sprite2D\" parent=\"{}\"]\nz_index = {}\nposition = Vector2({}, {})\nscale = Vector2({}, {})\ntexture = SubResource(\"AtlasTexture_{}\")\ncentered = false\n\n",
            rig.bone_path(&p.bone),
            p.z,
            fnum(p.bbox[0] - at[0]),
            fnum(p.bbox[1] - at[1]),
            fnum(1.0 / scale),
            fnum(1.0 / scale),
            p.name
        ));
    }
    let autoplay = rig
        .actions
        .first()
        .map(|a| format!("autoplay = &\"{}\"\n", a.name))
        .unwrap_or_default();
    nodes.push_str(&format!(
        "[node name=\"AnimationPlayer\" type=\"AnimationPlayer\" parent=\".\"]\nlibraries = {{\n&\"\": SubResource(\"AnimationLibrary_{name}\")\n}}\n{autoplay}"
    ));
    format!(
        "[gd_scene load_steps={} format=3]\n\n{res}{nodes}",
        subs + 2
    )
}

// ---------------------------------------------------------------------------
// Poses (`rigpose.go`): la interpolación de Godot (cúbica entre claves y
// circular en los bucles), para que lo que ven el agente y el studio sea lo
// que reproduce el juego.
// ---------------------------------------------------------------------------

/// x' = a x + c y + e, y' = b x + d y + f.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Mat {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Mat {
    const IDENTITY: Mat = Mat {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    const ZERO: Mat = Mat {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };

    fn mul(self, n: Mat) -> Mat {
        let m = self;
        Mat {
            a: m.a * n.a + m.c * n.b,
            b: m.b * n.a + m.d * n.b,
            c: m.a * n.c + m.c * n.d,
            d: m.b * n.c + m.d * n.d,
            e: m.a * n.e + m.c * n.f + m.e,
            f: m.b * n.e + m.d * n.f + m.f,
        }
    }
}

/// El valor de las claves (tiempos crecientes) en `t`, como la
/// interpolación cúbica de Godot.
fn cubic_at(times: &[f64], vals: &[f64], t: f64, looping: bool) -> f64 {
    let n = times.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 || t <= times[0] {
        return vals[0];
    }
    if t >= times[n - 1] {
        return vals[n - 1];
    }
    let mut i = 0;
    while i < n - 2 && times[i + 1] <= t {
        i += 1;
    }
    let at = |k: isize| -> f64 {
        let n = n as isize;
        if k < 0 && looping && n > 2 {
            // En un bucle, la última clave es igual que la primera.
            vals[(n - 2 + k + 1) as usize]
        } else if k < 0 {
            vals[0]
        } else if k >= n && looping && n > 2 {
            vals[(k - n + 1) as usize]
        } else if k >= n {
            vals[(n - 1) as usize]
        } else {
            vals[k as usize]
        }
    };
    let i = i as isize;
    let (p0, p1, p2, p3) = (at(i - 1), vals[i as usize], vals[i as usize + 1], at(i + 2));
    let u = (t - times[i as usize]) / (times[i as usize + 1] - times[i as usize]);
    0.5 * (2.0 * p1
        + (-p0 + p2) * u
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u * u
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u * u * u)
}

impl Rig {
    /// Para cada hueso, la transformación que lleva el reposo a la pose del
    /// instante `t` (segundos) de la acción.
    fn world_at(&self, a: &RigAction, t: f64) -> BTreeMap<String, Mat> {
        let rest: BTreeMap<String, [f64; 2]> = self
            .bones
            .iter()
            .map(|b| (b.name.clone(), [0.0, 0.0]))
            .collect();
        let (tracks, _) = self.tracks(a, &rest);
        let mut out: BTreeMap<String, Mat> = BTreeMap::new();
        for b in &self.bones {
            let (mut rot, mut dx, mut dy, mut sx, mut sy) = (0.0, 0.0, 0.0, 1.0, 1.0);
            if let Some(tr) = tracks.get(&b.name) {
                rot = cubic_at(&tr.times, &tr.rot, t, a.looping);
                let xs: Vec<f64> = tr.pos.iter().map(|p| p[0]).collect();
                let ys: Vec<f64> = tr.pos.iter().map(|p| p[1]).collect();
                dx = cubic_at(&tr.times, &xs, t, a.looping);
                dy = cubic_at(&tr.times, &ys, t, a.looping);
                if tr.scale_used {
                    let xs: Vec<f64> = tr.scale.iter().map(|p| p[0]).collect();
                    let ys: Vec<f64> = tr.scale.iter().map(|p| p[1]).collect();
                    sx = cubic_at(&tr.times, &xs, t, a.looping);
                    sy = cubic_at(&tr.times, &ys, t, a.looping);
                }
            }
            let (c, s) = (cos(rot), sin(rot));
            let (x, y) = (b.at[0], b.at[1]);
            // Se mueve en el espacio del padre, y gira y escala alrededor de
            // la articulación (rotación por escala, como Godot).
            let (ma, mb, mc, md) = (c * sx, s * sx, -s * sy, c * sy);
            let local = Mat {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: dx,
                f: dy,
            }
            .mul(Mat {
                a: ma,
                b: mb,
                c: mc,
                d: md,
                e: x - ma * x - mc * y,
                f: y - mb * x - md * y,
            });
            let parent = out.get(&b.parent).copied().unwrap_or(Mat::IDENTITY);
            out.insert(b.name.clone(), parent.mul(local));
        }
        out
    }

    /// La opacidad de cada hueso en el instante `t` (segundos), ya
    /// multiplicada por la de sus padres.
    fn alpha_world(&self, a: &RigAction, t: f64) -> BTreeMap<String, f64> {
        let rest: BTreeMap<String, [f64; 2]> = self
            .bones
            .iter()
            .map(|b| (b.name.clone(), [0.0, 0.0]))
            .collect();
        let (tracks, _) = self.tracks(a, &rest);
        let mut out: BTreeMap<String, f64> = BTreeMap::new();
        for b in &self.bones {
            let own = match tracks.get(&b.name) {
                Some(tr) if tr.alpha_used => {
                    cubic_at(&tr.times, &tr.alpha, t, a.looping).clamp(0.0, 1.0)
                }
                _ => 1.0,
            };
            let parent = out.get(&b.parent).copied().unwrap_or(1.0);
            out.insert(b.name.clone(), parent * own);
        }
        out
    }

    /// La caja de las cajas de todas las piezas movidas por su hueso.
    fn posed_bounds(&self, world: &BTreeMap<String, Mat>) -> (f64, f64, f64, f64) {
        let (mut x0, mut y0, mut x1, mut y1) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in &self.parts {
            let m = world.get(&p.bone).copied().unwrap_or(Mat::ZERO);
            let b = p.bbox;
            for c in [
                [b[0], b[1]],
                [b[0] + b[2], b[1]],
                [b[0], b[1] + b[3]],
                [b[0] + b[2], b[1] + b[3]],
            ] {
                let x = m.a * c[0] + m.c * c[1] + m.e;
                let y = m.b * c[0] + m.d * c[1] + m.f;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
        (x0, y0, x1, y1)
    }

    /// Cada pieza con la transformación de su hueso, en orden de apilado.
    fn pose_markup(&self, world: &BTreeMap<String, Mat>, alpha: &BTreeMap<String, f64>) -> String {
        let mut b = String::new();
        for p in &self.parts {
            let m = world.get(&p.bone).copied().unwrap_or(Mat::ZERO);
            let op = alpha.get(&p.bone).copied().unwrap_or(1.0);
            if op <= 0.0 {
                continue;
            }
            let opacity = if op < 1.0 {
                format!(r#" opacity="{}""#, fnum(op))
            } else {
                String::new()
            };
            b.push_str(&format!(
                r#"<g transform="matrix({} {} {} {} {} {})"{opacity}>{}</g>"#,
                fnum(m.a),
                fnum(m.b),
                fnum(m.c),
                fnum(m.d),
                fnum(m.e),
                fnum(m.f),
                p.markup
            ));
        }
        b
    }
}

impl Rig {
    /// La matriz `[a, b, c, d, e, f]` de cada hueso en el segundo `seconds`
    /// de la acción (sin acción: el reposo, la identidad). Un punto `(x, y)`
    /// del reposo queda en `(a·x + c·y + e, b·x + d·y + f)`.
    pub fn pose_at(&self, action: Option<&RigAction>, seconds: f64) -> BTreeMap<String, [f64; 6]> {
        let world = match action {
            Some(a) => self.world_at(a, seconds),
            None => self
                .bones
                .iter()
                .map(|b| (b.name.clone(), Mat::IDENTITY))
                .collect(),
        };
        world
            .into_iter()
            .map(|(name, m)| (name, [m.a, m.b, m.c, m.d, m.e, m.f]))
            .collect()
    }

    /// La opacidad de cada hueso (de 0 a 1, con la de sus padres) en el
    /// segundo `seconds` de la acción; sin acción, o si no la cambia, 1.
    pub fn alpha_at(&self, action: Option<&RigAction>, seconds: f64) -> BTreeMap<String, f64> {
        match action {
            Some(a) => self.alpha_world(a, seconds),
            None => BTreeMap::new(),
        }
    }

    /// El dibujo entero posado en el segundo `seconds` de la acción: lo
    /// fijo (`still`, el contenido sin piezas) y encima cada pieza movida por
    /// su hueso, en su orden. Para exportar fotogramas.
    pub fn posed_svg(&self, action: Option<&RigAction>, seconds: f64, still: &str) -> String {
        let world = match action {
            Some(a) => self.world_at(a, seconds),
            None => self
                .bones
                .iter()
                .map(|b| (b.name.clone(), Mat::IDENTITY))
                .collect(),
        };
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><defs>{defs}</defs>{still}{parts}</svg>"#,
            w = self.width,
            h = self.height,
            defs = self.defs,
            parts = self.pose_markup(&world, &self.alpha_at(action, seconds))
        )
    }

    /// Una pieza sola como SVG del tamaño del dibujo (para rasterizarla
    /// recortada a su caja).
    pub fn part_svg(&self, part: &RigPart) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><defs>{defs}</defs>{markup}</svg>"#,
            w = self.width,
            h = self.height,
            defs = self.defs,
            markup = part.markup
        )
    }
}

/// Lo que hay que rasterizar en `vec poses`.
#[derive(Debug, Clone, PartialEq)]
pub struct PosesSvg {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub out_width: u32,
}

/// Cada acción como una fila de fotogramas (primero el reposo), sobre un
/// damero claro, para leer el movimiento de un vistazo (`PosesPNG`).
pub fn poses_svg(
    rig: Option<&Rig>,
    only: &str,
    frames: i64,
    width: i64,
) -> Result<PosesSvg, String> {
    let rig = match rig {
        Some(r) if !r.parts.is_empty() => r,
        _ => return Err("el dibujo no tiene esqueleto con piezas".into()),
    };
    let frames = frames.clamp(2, 24).max(2) as usize;
    let rows: Vec<&RigAction> = rig
        .actions
        .iter()
        .filter(|a| only.is_empty() || a.name == only)
        .collect();
    if !only.is_empty() && rows.is_empty() {
        return Err(format!(
            "no hay ninguna acción llamada {}",
            crate::error::go_quote(only)
        ));
    }
    let rest_world: BTreeMap<String, Mat> = rig
        .bones
        .iter()
        .map(|b| (b.name.clone(), Mat::IDENTITY))
        .collect();
    let time_at = |a: &RigAction, f: usize| {
        if a.looping {
            a.duration * f as f64 / frames as f64
        } else {
            a.duration * f as f64 / (frames - 1) as f64
        }
    };
    // La celda: caben todas las poses, para que ningún paso quede cortado.
    let (mut x0, mut y0, mut x1, mut y1) = rig.posed_bounds(&rest_world);
    for a in &rows {
        for f in 0..frames {
            let (a0, b0, a1, b1) = rig.posed_bounds(&rig.world_at(a, time_at(a, f)));
            x0 = x0.min(a0);
            y0 = y0.min(b0);
            x1 = x1.max(a1);
            y1 = y1.max(b1);
        }
    }
    y1 = y1.max(rig.origin[1]);
    let (pad_x, pad_y) = ((x1 - x0) * 0.06, (y1 - y0) * 0.05);
    let (cx, cy, cw, ch) = (
        x0 - pad_x,
        y0 - pad_y,
        x1 - x0 + 2.0 * pad_x,
        y1 - y0 + 2.0 * pad_y,
    );
    let label = ch * 0.07;
    let tw = cw * frames as f64;
    let th = (ch + label) * (rows.len() + 1) as f64;
    let mut b = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {tw} {th}" width="{tw}" height="{th}"><defs>{defs}<pattern id="pxck" width="{l}" height="{l}" patternUnits="userSpaceOnUse"><rect width="{l}" height="{l}" fill="#e9ecef"/><rect width="{h}" height="{h}" fill="#dde1e6"/><rect x="{h}" y="{h}" width="{h}" height="{h}" fill="#dde1e6"/></pattern></defs><rect width="100%" height="100%" fill="url(#pxck)"/>"##,
        tw = num(tw),
        th = num(th),
        defs = rig.defs,
        l = num(label),
        h = num(label / 2.0)
    );
    let mut cell = |col: usize,
                    row: usize,
                    world: &BTreeMap<String, Mat>,
                    alpha: &BTreeMap<String, f64>,
                    text: &str| {
        let (x, y) = (col as f64 * cw, row as f64 * (ch + label));
        b.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="{}" font-family="DejaVu Sans, sans-serif" fill="#333">{text}</text>"##,
            num(x + label * 0.3),
            num(y + label * 0.8),
            num(label * 0.6)
        ));
        let ground = y + label + rig.origin[1] - cy;
        b.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#9aa3ad" stroke-width="{}"/>"##,
            num(x),
            num(ground),
            num(x + cw),
            num(ground),
            num(label / 12.0)
        ));
        b.push_str(&format!(
            r#"<svg x="{}" y="{}" width="{}" height="{}" viewBox="{} {} {} {}">{}</svg>"#,
            num(x),
            num(y + label),
            num(cw),
            num(ch),
            num(cx),
            num(cy),
            num(cw),
            num(ch),
            rig.pose_markup(world, alpha)
        ));
    };
    cell(0, 0, &rest_world, &BTreeMap::new(), "reposo del dibujo");
    for (i, a) in rows.iter().enumerate() {
        for f in 0..frames {
            let t = time_at(a, f);
            let mut text = format!("{:.2}", t / a.duration);
            if f == 0 {
                text = format!("{} {text}", a.name);
            }
            cell(f, i + 1, &rig.world_at(a, t), &rig.alpha_world(a, t), &text);
        }
    }
    b.push_str("</svg>");
    let out_width = if width <= 0 {
        2400.min(tw as i64)
    } else {
        width
    };
    Ok(PosesSvg {
        svg: b,
        width: tw.ceil(),
        height: th.ceil(),
        out_width: out_width.max(0) as u32,
    })
}
