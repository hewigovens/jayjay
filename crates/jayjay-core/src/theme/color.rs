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

pub fn contrast_ratio(a: u32, b: u32) -> f32 {
    let la = luminance(a);
    let lb = luminance(b);
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

/// Moves `color` toward `toward`, then on to whichever of black or white scores better, in small steps until it meets every `(background, min_ratio)` floor.
pub fn readable_on(color: u32, toward: u32, floors: &[(u32, f32)]) -> u32 {
    let score = |candidate: u32| {
        floors
            .iter()
            .map(|(background, min_ratio)| contrast_ratio(candidate, *background) / min_ratio)
            .fold(f32::INFINITY, f32::min)
    };
    let extreme = if score(0xffffff) >= score(0x000000) {
        0xffffff
    } else {
        0x000000
    };
    (0..=20)
        .map(|step| mix(color, toward, step as f32 / 20.))
        .chain((1..=20).map(|step| mix(toward, extreme, step as f32 / 20.)))
        .find(|candidate| score(*candidate) >= 1.)
        .unwrap_or(extreme)
}
