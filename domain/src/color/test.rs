use crate::color::{
    ApproxBrightness,
    Brightness,
    CmyColor,
    Color2,
    ContrastFgWithBg,
    GrayColor,
    RgbColor,
    UpdateBg,
    UpdateFg,
    Updater,
};

#[test]
fn gray_color_brightness() {
    assert_eq!(GrayColor::new(0).approx_brightness(), Brightness { level: 0 });
    assert_eq!(
        GrayColor::new(3).approx_brightness(),
        Brightness { level: 8548 }
    );
    assert_eq!(
        GrayColor::new(9).approx_brightness(),
        Brightness { level: 25644 }
    );
    assert_eq!(
        GrayColor::new(12).approx_brightness(),
        Brightness { level: 34192 }
    );
    assert_eq!(
        GrayColor::new(23).approx_brightness(),
        Brightness { level: 65535 }
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
        Brightness { level: 26214 }
    );
    assert_eq!(
        CmyColor::new(5, 5, 5).approx_brightness(),
        Brightness { level: 65535 }
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
        Brightness { level: 23644 }
    );
    assert_eq!(
        RgbColor { red: 255, green: 255, blue: 255 }.approx_brightness(),
        Brightness { level: 65535 }
    );
}

#[test]
fn updaters() {
    let updater = (
        UpdateFg(CmyColor::new(1, 2, 3).into()),
        ContrastFgWithBg,
        UpdateBg(CmyColor::new(4, 4, 5).into()),
    );

    let pair = Color2 {
        foreground: RgbColor { red: 0, green: 0, blue: 0 }.into(),
        background: CmyColor::new(1, 1, 1).into(),
    };

    assert_eq!(
        updater.update(pair),
        Color2 {
            foreground: CmyColor::new(2, 3, 5).into(),
            background: CmyColor::new(4, 4, 5).into(),
        }
    );
}
