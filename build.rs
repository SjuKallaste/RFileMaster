extern crate winres;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon/icon.ico");
        res.compile().expect("Failed to compile Windows resource file");
    }
}