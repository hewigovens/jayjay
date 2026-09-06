fn channels(color: u32) -> [f32; 3] {
    [
        ((color >> 16) & 0xff) as f32,
        ((color >> 8) & 0xff) as f32,
        (color & 0xff) as f32,
    ]
}

fn from_channels(rgb: [f32; 3]) -> u32 {
    let [r, g, b] = rgb.map(|c| c.round().clamp(0., 255.) as u32);
    (r << 16) | (g << 8) | b
}

pub fn mix(from: u32, to: u32, amount: f32) -> u32 {
    let amount = amount.clamp(0., 1.);
    let a = channels(from);
    let b = channels(to);
    from_channels([
        a[0] + (b[0] - a[0]) * amount,
        a[1] + (b[1] - a[1]) * amount,
        a[2] + (b[2] - a[2]) * amount,
    ])
}

/// WCAG relative luminance in `0..=1`.
pub fn luminance(color: u32) -> f32 {
    let linear = |c: f32| {
        let c = c / 255.;
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = channels(color).map(linear);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn is_dark(background: u32) -> bool {
    luminance(background) < 0.4
}
