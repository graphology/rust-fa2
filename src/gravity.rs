use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_gravity<F: Float>(
    settings: &FA2Settings<F>,
    xs: &[F],
    ys: &[F],
    ms: &[F],
    out_xs: &mut [F],
    out_ys: &mut [F],
) {
    let g = settings.gravity / settings.scaling_ratio;

    for i in 0..xs.len() {
        let mut factor = F::zero();

        let x = xs[i];
        let y = ys[i];
        let mass = ms[i];

        let distance = (x.powi(2) + y.powi(2)).sqrt();

        // TODO: make this branchless
        if distance > F::zero() {
            factor = settings.scaling_ratio * mass * g;

            // TODO: make this branchless
            if !settings.strong_gravity_mode {
                factor /= distance;
            }
        }

        out_xs[i] -= x * factor;
        out_ys[i] -= y * factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_gravity() {
        let settings = FA2Settings::<f32>::default();

        let xs = [1.0, 2.0];
        let ys = [3.0, -5.0];
        let ms = [1.0, 1.5];
        let mut out_xs = [1.0, 2.0];
        let mut out_ys = [3.0, -5.0];

        apply_gravity(&settings, &xs, &ys, &ms, &mut out_xs, &mut out_ys);

        assert_eq!(out_xs, [0.6837722, 1.442914]);
        assert_eq!(out_ys, [2.0513167, -3.607285]);
    }
}
