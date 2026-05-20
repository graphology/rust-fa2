use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_attraction<F: Float>(
    settings: &FA2Settings<F>,
    nodes: &[F],
    edges: &[(usize, usize, F)],
    out: &mut [F],
) {
    // TODO: plug outbound_attraction_distribution
    let coefficient = F::one();

    for (source, target, weight) in edges {
        let ewc = weight.powf(settings.edge_weight_influence);
        let factor = -coefficient * ewc;

        let x_dist = nodes[source * 3] - nodes[target * 3];
        let y_dist = nodes[source * 3 + 1] - nodes[target * 3 + 1];

        out[source * 2] += x_dist * factor;
        out[source * 2 + 1] += y_dist * factor;

        out[target * 2] -= x_dist * factor;
        out[target * 2 + 1] -= y_dist * factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_attraction() {
        let settings = FA2Settings::<f32>::default();

        let nodes = [1.0, 3.0, 1.0, 2.0, -5.0, 1.5];
        let edges = [(0, 1, 1.0)];
        let mut out = [1.0, 3.0, 2.0, -5.0];

        apply_attraction(&settings, &nodes, &edges, &mut out);

        assert_eq!(out, [2.0, -5.0, 1.0, 3.0]);
    }
}
