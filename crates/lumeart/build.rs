//! Al compilar para Windows, el `.exe` lleva el icono de LumeArt
//! (`assets/brand/icon.ico`) y su nombre. En Linux y macOS no hace nada: la
//! ventana pone su icono al abrirse.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/brand/icon.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/brand/icon.ico")
            .set("ProductName", "LumeArt")
            .set(
                "FileDescription",
                "LumeArt: studio de arte para videojuegos con agente de IA",
            );
        if let Err(e) = res.compile() {
            println!("cargo:warning=no se pudo poner el icono al .exe: {e}");
        }
    }
}
