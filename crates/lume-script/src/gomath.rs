//! `Math` de QuickJS con las funciones de goja
//! ([`lume_domain::gomath`]), para que los scripts den los mismos números
//! que en pxcli.

use lume_domain::gomath::{acos, asin, atan, atan2, cos, exp, hypot, log, pow, sin, tan};

/// Una función de un número y su nombre en `Math`.
type Unary = (&'static str, fn(f64) -> f64);
/// Una función de dos números y su nombre en `Math`.
type Binary = (&'static str, fn(f64, f64) -> f64);

/// Cambia las funciones de `Math` de QuickJS por estas (las demás ya dan
/// lo mismo: redondeos, raíz cuadrada, máximos…).
pub fn install<'js>(ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
    use rquickjs::function::Rest;
    use rquickjs::{Coerced, FromJs, Function, Object, Value};

    fn number<'js>(
        ctx: &rquickjs::Ctx<'js>,
        args: &[Value<'js>],
        i: usize,
    ) -> rquickjs::Result<f64> {
        match args.get(i) {
            Some(v) => Coerced::<f64>::from_js(ctx, v.clone()).map(|c| c.0),
            None => Ok(f64::NAN),
        }
    }
    let math: Object = ctx.globals().get("Math")?;
    let one: [Unary; 8] = [
        ("sin", sin),
        ("cos", cos),
        ("tan", tan),
        ("atan", atan),
        ("asin", asin),
        ("acos", acos),
        ("exp", exp),
        ("log", log),
    ];
    for (name, func) in one {
        math.set(
            name,
            Function::new(
                ctx.clone(),
                move |ctx: rquickjs::Ctx<'js>, args: Rest<Value<'js>>| {
                    number(&ctx, &args.0, 0).map(func)
                },
            )?,
        )?;
    }
    let two: [Binary; 2] = [("atan2", atan2), ("pow", pow)];
    for (name, func) in two {
        math.set(
            name,
            Function::new(
                ctx.clone(),
                move |ctx: rquickjs::Ctx<'js>, args: Rest<Value<'js>>| {
                    Ok::<_, rquickjs::Error>(func(
                        number(&ctx, &args.0, 0)?,
                        number(&ctx, &args.0, 1)?,
                    ))
                },
            )?,
        )?;
    }
    math.set(
        "hypot",
        Function::new(
            ctx.clone(),
            |ctx: rquickjs::Ctx<'js>, args: Rest<Value<'js>>| {
                let values = (0..args.0.len())
                    .map(|i| number(&ctx, &args.0, i))
                    .collect::<rquickjs::Result<Vec<f64>>>()?;
                Ok::<_, rquickjs::Error>(hypot(&values))
            },
        )?,
    )?;
    Ok(())
}
