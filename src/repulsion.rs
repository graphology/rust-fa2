use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_pairwise_repulsion<F: Float>(settings: &FA2Settings<F>, nodes: &[F], out: &mut [F]) {
    let order = nodes.len() / 3;

    for n1 in 0..order {
        let o1 = n1 * 3;

        let x1 = nodes[o1];
        let y1 = nodes[o1 + 1];
        let m1 = nodes[o1 + 2];

        for n2 in (n1 + 1)..order {
            let o2 = n2 * 3;
            let x2 = nodes[o2];
            let y2 = nodes[o2 + 1];
            let m2 = nodes[o2 + 2];

            let x_dist = x1 - x2;
            let y_dist = y1 - y2;

            let distance = (x_dist.powi(2) + y_dist.powi(2)).sqrt();

            // TODO: optimize away this branch
            if distance > F::zero() {
                let factor = (settings.scaling_ratio * m1 * m2) / distance / distance;

                out[n1 * 2] += x_dist * factor;
                out[n1 * 2 + 1] += y_dist * factor;

                out[n2 * 2] -= x_dist * factor;
                out[n2 * 2 + 1] -= y_dist * factor;
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

        let nodes = [1.0, 3.0, 1.0, 2.0, -5.0, 1.5];
        let mut out = [1.0, 3.0, 2.0, -5.0];

        apply_pairwise_repulsion(&settings, &nodes, &mut out);

        assert_eq!(out, [0.97692305, 3.1846154, 2.023077, -5.1846156]);
    }
}
