macro_rules! base_path {
    () => {
        concat!(
            "..",
            host_path_separator!(),
            "..",
            host_path_separator!(),
            "assets",
            host_path_separator!(),
            "fonts",
        )
    };
}

macro_rules! load_font {
    ($font_name:literal, $variant:literal) => {{
        let font_data = include_bytes!(concat!(
            base_path!(),
            host_path_separator!(),
            $font_name,
            host_path_separator!(),
            concat!($font_name, "_", $variant, ".ttf"),
        ));

        Font::try_from_bytes(font_data).expect(concat!(
            "font '",
            concat!($font_name, "_", $variant, ".ttf"),
            "' should parse correctly"
        ))
    }};
}

use rusttype::Font;

pub struct Lato;

impl Lato {
    pub fn bold() -> Font<'static> {
        load_font!("lato", "700")
    }
}
