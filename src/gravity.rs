use crate::settings::Settings;
use crate::traits::Float;

pub fn apply_gravity<F: Float>(settings: &Settings<F>, nodes: &[F], out: &mut [F]) {
    let g = settings.gravity / settings.scaling_ratio;

    for (node, out_node) in nodes.chunks(3).zip(out.chunks_mut(2)) {
        let mut factor = F::zero();

        let x = node[0];
        let y = node[1];
        let mass = node[2];

        let distance = (x.powi(2) + y.powi(2)).sqrt();

        if distance > F::zero() {
            factor = settings.scaling_ratio * mass * g;

            // TODO: make this branchless
            if !settings.strong_gravity_mode {
                factor /= distance;
            }
        }

        out_node[0] -= x * factor;
        out_node[1] -= y * factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_gravity() {
        let settings = Settings::<f32>::default();

        let nodes = [1.0, 3.0, 1.0, 2.0, -5.0, 1.5];
        let mut out = [1.0, 3.0, 2.0, -5.0];

        apply_gravity(&settings, &nodes, &mut out);

        assert_eq!(out, [0.6837722, 2.0513167, 1.442914, -3.607285])
    }
}
