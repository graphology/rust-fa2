use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_attraction<F: Float>(
    settings: &FA2Settings<F>,
    xs: &[F],
    ys: &[F],
    edges: &[(usize, usize, F)],
    out_xs: &mut [F],
    out_ys: &mut [F],
) {
    for (source, target, weight) in edges.iter().copied() {
        let factor = -weight.powf(settings.edge_weight_influence);

        let x_dist = xs[source] - xs[target];
        let y_dist = ys[source] - ys[target];

        let x_displacement = x_dist * factor;
        let y_displacement = y_dist * factor;

        out_xs[source] += x_displacement;
        out_ys[source] += y_displacement;

        out_xs[target] -= x_displacement;
        out_ys[target] -= y_displacement;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_attraction() {
        let settings = FA2Settings::<f32>::default();

        let xs = [1.0, 2.0];
        let ys = [3.0, -5.0];
        let edges = [(0, 1, 1.0)];
        let mut out_xs = [1.0, 2.0];
        let mut out_ys = [3.0, -5.0];

        apply_attraction(&settings, &xs, &ys, &edges, &mut out_xs, &mut out_ys);

        assert_eq!(out_xs, [2.0, 1.0]);
        assert_eq!(out_ys, [-5.0, 3.0])
    }
}
