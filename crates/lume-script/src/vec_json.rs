//! El esqueleto (`__rigInfo`) y la escena (`__sceneInfo`) que da `lib.js`
//! en JSON, leídos como `json.Unmarshal` de Go en pxcli: lo que falta o es
//! `null` queda a cero, un tipo equivocado invalida todo y en un array de
//! tamaño fijo sobran o faltan elementos sin error.

use std::collections::BTreeMap;

use lume_domain::vec::rig::{Rig, RigAction, RigBone, RigKey, RigPart, RigPose};
use lume_domain::vec::scene::{Scene, SceneBackground, SceneLayer, ScenePiece};
use serde_json::{Map, Value};

type R<T> = Result<T, ()>;

fn absent(v: Option<&Value>) -> bool {
    matches!(v, None | Some(Value::Null))
}

fn number(v: Option<&Value>) -> R<f64> {
    if absent(v) {
        return Ok(0.0);
    }
    v.and_then(Value::as_f64).ok_or(())
}

fn int(v: Option<&Value>) -> R<i64> {
    if absent(v) {
        return Ok(0);
    }
    v.and_then(Value::as_i64).ok_or(())
}

fn text(v: Option<&Value>) -> R<String> {
    if absent(v) {
        return Ok(String::new());
    }
    v.and_then(Value::as_str).map(str::to_string).ok_or(())
}

fn boolean(v: Option<&Value>) -> R<bool> {
    if absent(v) {
        return Ok(false);
    }
    v.and_then(Value::as_bool).ok_or(())
}

fn fixed<const N: usize>(v: Option<&Value>) -> R<[f64; N]> {
    let mut out = [0.0; N];
    if absent(v) {
        return Ok(out);
    }
    let items = v.and_then(Value::as_array).ok_or(())?;
    // Los de más se descartan sin mirarlos.
    for (slot, item) in out.iter_mut().zip(items) {
        *slot = number(Some(item))?;
    }
    Ok(out)
}

fn optional_pair(v: Option<&Value>) -> R<Option<[f64; 2]>> {
    if absent(v) {
        return Ok(None);
    }
    fixed::<2>(v).map(Some)
}

fn list(v: Option<&Value>) -> R<&[Value]> {
    if absent(v) {
        return Ok(&[]);
    }
    v.and_then(Value::as_array).map(Vec::as_slice).ok_or(())
}

/// Un objeto (un `null` es un objeto vacío, como un struct en Go).
fn object(v: &Value) -> R<Option<&Map<String, Value>>> {
    match v {
        Value::Null => Ok(None),
        Value::Object(map) => Ok(Some(map)),
        _ => Err(()),
    }
}

fn field<'a>(map: Option<&'a Map<String, Value>>, key: &str) -> Option<&'a Value> {
    map.and_then(|m| m.get(key))
}

fn rig(v: &Value) -> R<Rig> {
    let m = object(v)?;
    let mut out = Rig {
        origin: fixed(field(m, "origin"))?,
        width: int(field(m, "width"))?,
        height: int(field(m, "height"))?,
        defs: text(field(m, "defs"))?,
        ..Rig::default()
    };
    for b in list(field(m, "bones"))? {
        let b = object(b)?;
        out.bones.push(RigBone {
            name: text(field(b, "name"))?,
            parent: text(field(b, "parent"))?,
            at: fixed(field(b, "at"))?,
            line: int(field(b, "line"))?,
        });
    }
    for a in list(field(m, "actions"))? {
        let a = object(a)?;
        let mut keys = Vec::new();
        for k in list(field(a, "keys"))? {
            let k = object(k)?;
            let mut pose = BTreeMap::new();
            if let Some(p) = field(k, "pose")
                && let Some(map) = object(p)?
            {
                for (bone, value) in map {
                    let v = object(value)?;
                    let or_one = |key: &str| -> R<f64> {
                        if absent(field(v, key)) {
                            Ok(1.0)
                        } else {
                            number(field(v, key))
                        }
                    };
                    pose.insert(
                        bone.clone(),
                        RigPose {
                            rot: number(field(v, "rot"))?,
                            x: number(field(v, "x"))?,
                            y: number(field(v, "y"))?,
                            sx: or_one("sx")?,
                            sy: or_one("sy")?,
                            alpha: or_one("alpha")?,
                        },
                    );
                }
            }
            keys.push(RigKey {
                t: number(field(k, "t"))?,
                pose,
            });
        }
        out.actions.push(RigAction {
            name: text(field(a, "name"))?,
            duration: number(field(a, "duration"))?,
            looping: boolean(field(a, "loop"))?,
            movement: number(field(a, "move"))?,
            keys,
            line: int(field(a, "line"))?,
        });
    }
    for p in list(field(m, "parts"))? {
        let p = object(p)?;
        out.parts.push(RigPart {
            name: text(field(p, "name"))?,
            bone: text(field(p, "bone"))?,
            z: int(field(p, "z"))?,
            bbox: fixed(field(p, "box"))?,
            markup: text(field(p, "markup"))?,
        });
    }
    Ok(out)
}

/// `parseRig`: nada si el texto está vacío, no se entiende o no tiene
/// huesos.
pub fn parse_rig(json: &str) -> Option<Rig> {
    if json.is_empty() {
        return None;
    }
    let value: Value = serde_json::from_str(json).ok()?;
    rig(&value).ok().filter(|r| !r.bones.is_empty())
}

fn scene(v: &Value) -> R<Scene> {
    let m = object(v)?;
    let background = match field(m, "background") {
        None | Some(Value::Null) => None,
        Some(bg) => {
            let b = object(bg)?;
            Some(SceneBackground {
                color: text(field(b, "color"))?,
                markup: text(field(b, "markup"))?,
            })
        }
    };
    let mut out = Scene {
        width: int(field(m, "width"))?,
        height: int(field(m, "height"))?,
        defs: text(field(m, "defs"))?,
        background,
        layers: Vec::new(),
    };
    for l in list(field(m, "layers"))? {
        let l = object(l)?;
        let mut pieces = Vec::new();
        for p in list(field(l, "pieces"))? {
            let p = object(p)?;
            pieces.push(ScenePiece {
                name: text(field(p, "name"))?,
                element: boolean(field(p, "element"))?,
                pivot: optional_pair(field(p, "pivot"))?,
                parallax: optional_pair(field(p, "parallax"))?,
                line: int(field(p, "line"))?,
                markup: text(field(p, "markup"))?,
            });
        }
        out.layers.push(SceneLayer {
            name: text(field(l, "name"))?,
            parallax: optional_pair(field(l, "parallax"))?,
            line: int(field(l, "line"))?,
            pieces,
        });
    }
    Ok(out)
}

/// `parseScene`: nada si el texto está vacío o no se entiende.
pub fn parse_scene(json: &str) -> Option<Scene> {
    if json.is_empty() {
        return None;
    }
    let value: Value = serde_json::from_str(json).ok()?;
    scene(&value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_go_unmarshal() {
        let r = parse_rig(
            r#"{"bones":[{"name":"torso","parent":"","at":[10,20],"line":3}],"actions":[{"name":"andar","duration":1,"loop":true,"move":0,"keys":[{"t":0,"pose":{"torso":{"rot":5,"x":null,"y":0}}}],"line":9}],"origin":[1,2,3],"width":100,"height":80,"defs":"","parts":[{"name":"torso","bone":"torso","z":0,"box":[1,2,3],"markup":"<g/>"}]}"#,
        )
        .unwrap();
        assert_eq!(r.bones[0].at, [10.0, 20.0]);
        assert_eq!(r.origin, [1.0, 2.0]);
        assert_eq!(r.parts[0].bbox, [1.0, 2.0, 3.0, 0.0]);
        assert_eq!(r.actions[0].keys[0].pose["torso"].rot, 5.0);
        assert!(r.actions[0].looping);
        // Sin huesos, vacío o con un tipo equivocado: nada.
        assert!(parse_rig(r#"{"bones":[]}"#).is_none());
        assert!(parse_rig("").is_none());
        assert!(parse_rig(r#"{"bones":[{"name":3}]}"#).is_none());
        assert!(parse_rig(r#"{"bones":[{"name":"a","line":1.5}]}"#).is_none());

        let s = parse_scene(
            r#"{"width":64,"height":32,"defs":"","background":null,"layers":[{"name":"cielo","parallax":null,"line":2,"pieces":[{"markup":"<g/>"},{"name":"nube","element":true,"pivot":[3,4],"parallax":null,"line":5,"markup":"<g/>"}]}]}"#,
        )
        .unwrap();
        assert!(s.background.is_none());
        assert_eq!(s.layers[0].parallax, None);
        assert_eq!(s.layers[0].pieces[1].pivot, Some([3.0, 4.0]));
        assert!(s.layers[0].pieces[1].element);
    }
}
