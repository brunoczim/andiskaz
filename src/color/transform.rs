//! This module defines color transformers. A color [`Transformer`] is simply a
//! function over [`Color`]s. In fact, if you want to use a plain function as a
//! [`Transformer`], use [`FromFn`]. Analogously, there is a transformer over
//! [`Color2`]s, the [`PairTransformer`].

use crate::color::{ApproxBrightness, Brightness, Color, Color2};

/// A color transformer. Changes a color.
pub trait Transformer {
    /// Given an input [`Color`], generates a new color to replace the input.
    fn transform(&self, color: Color) -> Color;
}

impl<'this, T> Transformer for &'this T
where
    T: Transformer,
{
    fn transform(&self, color: Color) -> Color {
        (**self).transform(color)
    }
}

/// A color pair transformer. Changes a color pair.
pub trait PairTransformer {
    /// Given an input [`Color2`], generates a new color pair to replace the
    /// input.
    fn transform_pair(&self, colors: Color2) -> Color2;
}

impl<'this, T> PairTransformer for &'this T
where
    T: PairTransformer,
{
    fn transform_pair(&self, colors: Color2) -> Color2 {
        (**self).transform_pair(colors)
    }
}

/// Identity transformer. Preserves the input [`Color`] or [`Color2`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Id;

impl Transformer for Id {
    fn transform(&self, color: Color) -> Color {
        color
    }
}

impl PairTransformer for Id {
    fn transform_pair(&self, colors: Color2) -> Color2 {
        colors
    }
}

/// Sequence transformer. Applies the two given transformers in sequence on a
/// color. Similar to function composition, but `Seq(t, u)` is analogous to
/// `u(t(color))` **instead of** `t(u(color))` (the last one is how composition
/// usually works).
#[derive(Debug, Clone, Copy, Default)]
pub struct Seq<T, U>(
    /// This is the first transformer applied.
    pub T,
    /// This is the second transformer applied.
    pub U,
);

impl<T, U> Transformer for Seq<T, U>
where
    T: Transformer,
    U: Transformer,
{
    fn transform(&self, color: Color) -> Color {
        self.1.transform(self.0.transform(color))
    }
}

impl<T, U> PairTransformer for Seq<T, U>
where
    T: PairTransformer,
    U: PairTransformer,
{
    fn transform_pair(&self, colors: Color2) -> Color2 {
        self.1.transform_pair(self.0.transform_pair(colors))
    }
}

/// Creates a transformer from a given function if the function has a signature
/// similar to a [`Transformer`] or [`PairTransformer`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FromFn<F>(
    /// The function used to transform colors.
    pub F,
);

impl<F> Transformer for FromFn<F>
where
    F: Fn(Color) -> Color,
{
    fn transform(&self, color: Color) -> Color {
        (self.0)(color)
    }
}

impl<F> PairTransformer for FromFn<F>
where
    F: Fn(Color2) -> Color2,
{
    fn transform_pair(&self, colors: Color2) -> Color2 {
        (self.0)(colors)
    }
}

/// Replaces a single color entirely.
#[derive(Debug, Clone, Copy)]
pub struct Const(
    /// The new color.
    pub Color,
);

impl Transformer for Const {
    fn transform(&self, _color: Color) -> Color {
        self.0
    }
}

/// Inverts a single color.
#[derive(Debug, Clone, Copy, Default)]
pub struct Invert;

impl Transformer for Invert {
    fn transform(&self, color: Color) -> Color {
        !color
    }
}

/// Adapts a single color to **match** the given brightness.
#[derive(Debug, Clone, Copy)]
pub struct Adapt(
    /// Brightness of the new color.
    pub Brightness,
);

impl Transformer for Adapt {
    fn transform(&self, color: Color) -> Color {
        color.with_approx_brightness(self.0)
    }
}

/// Adapts a single color to **contrast** the given brightness.
#[derive(Debug, Clone, Copy)]
pub struct Contrast(
    /// Inverse brightness of the new color.
    pub Brightness,
);

impl Transformer for Contrast {
    fn transform(&self, color: Color) -> Color {
        color.with_approx_brightness(!self.0)
    }
}

/// Updates a [`Color2`] using the given foreground and background
/// [`Transformer`]s, each over a single color, by matching output foreground
/// with color produced by foreground transformer, and output background with
/// color produced by background transformer.
#[derive(Debug, Clone, Copy, Default)]
pub struct PairWith<F, B>
where
    F: Transformer,
    B: Transformer,
{
    /// The transformer of the foreground.
    pub foreground: F,
    /// The transformer of the background.
    pub background: B,
}

impl<F, B> PairTransformer for PairWith<F, B>
where
    F: Transformer,
    B: Transformer,
{
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            foreground: self.foreground.transform(colors.foreground),
            background: self.background.transform(colors.background),
        }
    }
}

/// Updates a [`Color2`] using the given foreground and background
/// [`Transformer`]s, each over a single color, like [`PairWith`], but swapping
/// foreground and background. This means input foreground is fed to a
/// transformer which produces background, and input background is fed to a
/// transformer which produces foreground.
#[derive(Debug, Clone, Copy, Default)]
pub struct SwapPairUsing<F, B>
where
    F: Transformer,
    B: Transformer,
{
    /// Foreground to background conversor.
    pub fg_to_bg: F,
    /// Background to foreground conversor.
    pub bg_to_fg: B,
}

/// Just swaps foreground with background and nothing more.
pub type SwapPair = SwapPairUsing<Id, Id>;

impl<F, B> PairTransformer for SwapPairUsing<F, B>
where
    F: Transformer,
    B: Transformer,
{
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            background: self.fg_to_bg.transform(colors.foreground),
            foreground: self.bg_to_fg.transform(colors.background),
        }
    }
}

/// Adapts foreground to **match** background.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdaptFgToBg;

impl PairTransformer for AdaptFgToBg {
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            foreground: colors
                .foreground
                .with_approx_brightness(colors.background.approx_brightness()),
            background: colors.background,
        }
    }
}

/// Adapts background to **match** foreground.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdaptBgToFg;

impl PairTransformer for AdaptBgToFg {
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            foreground: colors.foreground,
            background: colors
                .background
                .with_approx_brightness(colors.foreground.approx_brightness()),
        }
    }
}

/// Adapats foreground to **contrast** background.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContrastFgWithBg;

impl PairTransformer for ContrastFgWithBg {
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            foreground: colors
                .foreground
                .with_approx_brightness(!colors.background.approx_brightness()),
            background: colors.background,
        }
    }
}

/// Adapats background to **contrast** foreground.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContrastBgWithFg;

impl PairTransformer for ContrastBgWithFg {
    fn transform_pair(&self, colors: Color2) -> Color2 {
        Color2 {
            foreground: colors.foreground,
            background: colors
                .background
                .with_approx_brightness(!colors.foreground.approx_brightness()),
        }
    }
}
