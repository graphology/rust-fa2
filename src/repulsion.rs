use crate::settings::FA2Settings;
use crate::traits::Float;

#[allow(clippy::too_many_arguments)]
#[inline]
pub fn apply_nodewise_repulsion<F: Float>(
    settings: &FA2Settings<F>,
    x1: F,
    y1: F,
    m1: F,
    xs: &[F],
    ys: &[F],
    ms: &[F],
    out_x: &mut F,
    out_y: &mut F,
) {
    let order = xs.len();

    for n in 0..order {
        let x2 = xs[n];
        let y2 = ys[n];
        let m2 = ms[n];

        let x_dist = x1 - x2;
        let y_dist = y1 - y2;

        let distance = (x_dist.powi(2) + y_dist.powi(2)).sqrt();

        // TODO: optimize away this branch
        if distance > F::zero() {
            let factor = (settings.scaling_ratio * m1 * m2) / distance / distance;

            *out_x += x_dist * factor;
            *out_y += y_dist * factor;
        }
    }
}

pub fn apply_pairwise_repulsion<F: Float>(
    settings: &FA2Settings<F>,
    xs: &[F],
    ys: &[F],
    ms: &[F],
    out_xs: &mut [F],
    out_ys: &mut [F],
) {
    let order = xs.len();

    for n1 in 0..order {
        let x1 = xs[n1];
        let y1 = ys[n1];
        let m1 = ms[n1];

        for n2 in (n1 + 1)..order {
            let x2 = xs[n2];
            let y2 = ys[n2];
            let m2 = ms[n2];

            let x_dist = x1 - x2;
            let y_dist = y1 - y2;

            let distance = (x_dist.powi(2) + y_dist.powi(2)).sqrt();

            // TODO: optimize away this branch
            if distance > F::zero() {
                let factor = (settings.scaling_ratio * m1 * m2) / distance / distance;

                out_xs[n1] += x_dist * factor;
                out_ys[n1] += y_dist * factor;

                out_xs[n2] -= x_dist * factor;
                out_ys[n2] -= y_dist * factor;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_pairwise_repulsion() {
        let settings = FA2Settings::<f32>::default();

        let xs = [1.0, 2.0];
        let ys = [3.0, -5.0];
        let ms = [1.0, 1.5];
        let mut out_xs = [1.0, 2.0];
        let mut out_ys = [3.0, -5.0];

        apply_pairwise_repulsion(&settings, &xs, &ys, &ms, &mut out_xs, &mut out_ys);

        assert_eq!(out_xs, [0.97692305, 2.023077]);
        assert_eq!(out_ys, [3.1846154, -5.1846156]);
    }

    #[test]
    fn test_apply_nodewise_repulsion() {
        let settings = FA2Settings::<f32>::default();

        let xs = [1.0, 2.0];
        let ys = [3.0, -5.0];
        let ms = [1.0, 1.5];
        let mut out_xs = [1.0, 2.0];
        let mut out_ys = [3.0, -5.0];

        apply_nodewise_repulsion(
            &settings,
            xs[0],
            ys[0],
            ms[0],
            &xs,
            &ys,
            &ms,
            &mut out_xs[0],
            &mut out_ys[0],
        );

        assert_eq!(out_xs, [0.97692305, 2.0]);
        assert_eq!(out_ys, [3.1846154, -5.0]);
    }
}
