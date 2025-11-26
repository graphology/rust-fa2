use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_attraction<F: Float>(
    settings: &FA2Settings<F>,
    nodes: &[F],
    edges: &[(usize, usize, F)],
    out: &mut [F],
) {
    let coefficient = F::one();

    for (source, target, weight) in edges {
        let ewc = weight.powf(settings.edge_weight_influence);
        let factor = -coefficient * ewc;

        let x_dist = nodes[source * 3] - nodes[target * 3];
        let y_dist = nodes[source * 3 + 1] - nodes[target * 3 + 1];

        out[source * 2] += x_dist * factor;
        out[source * 2 + 1] += y_dist * factor;

        out[target * 2] += x_dist * factor;
        out[target * 2 + 1] += y_dist * factor;
    }
}
