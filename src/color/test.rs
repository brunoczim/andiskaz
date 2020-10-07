use crate::color::{
    transform,
    transform::Transformer,
    ApproxBrightness,
    Brightness,
    CmyColor,
    GrayColor,
    RgbColor,
};

#[test]
fn gray_color_brightness() {
    assert_eq!(GrayColor::new(0).approx_brightness(), Brightness { level: 0 });
    assert_eq!(GrayColor::new(3).approx_brightness(), Brightness { level: 33 });
    assert_eq!(
        GrayColor::new(9).approx_brightness(),
        Brightness { level: 100 }
    );
    assert_eq!(
        GrayColor::new(12).approx_brightness(),
        Brightness { level: 133 }
    );
    assert_eq!(
        GrayColor::new(23).approx_brightness(),
        Brightness { level: 255 }
    );
}

#[test]
fn cmy_color_brightness() {
    assert_eq!(
        CmyColor::new(0, 0, 0).approx_brightness(),
        Brightness { level: 0 }
    );
    assert_eq!(
        CmyColor::new(1, 2, 3).approx_brightness(),
        Brightness { level: 92 }
    );
    assert_eq!(
        CmyColor::new(5, 5, 5).approx_brightness(),
        Brightness { level: 255 }
    );
}

#[test]
fn rgb_color_brightness() {
    assert_eq!(
        RgbColor { red: 0, green: 0, blue: 0 }.approx_brightness(),
        Brightness { level: 0 }
    );
    assert_eq!(
        RgbColor { red: 51, green: 102, blue: 153 }.approx_brightness(),
        Brightness { level: 92 }
    );
    assert_eq!(
        RgbColor { red: 255, green: 255, blue: 255 }.approx_brightness(),
        Brightness { level: 255 }
    );
}

#[test]
fn transformers() {
    let transformer = transform::Seq(
        transform::Adapt(Brightness { level: 133 }),
        transform::Id,
    );
    assert_eq!(
        transformer.transform(GrayColor::new(3).into()),
        GrayColor::new(12).into()
    );
}
