use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_forces<F: Float>(
    settings: &FA2Settings<F>,
    nodes: &mut [F],
    deltas: &[F],
    last_deltas: &[F],
    convergences: &mut [F],
) -> F {
    let mut energy = F::zero();
    let two = F::from(2.0).unwrap();

    for (o1, convergence) in convergences.iter_mut().enumerate() {
        let o3 = o1 * 3;
        let o2 = o1 * 2;

        let x = nodes[o3];
        let y = nodes[o3 + 1];
        let mass = nodes[o3 + 2];

        let delta_x = deltas[o2];
        let delta_y = deltas[o2 + 1];

        let last_delta_x = last_deltas[o2];
        let last_delta_y = last_deltas[o2 + 1];

        let movement = ((last_delta_x - delta_x).powi(2) + (last_delta_y - delta_y).powi(2)).sqrt();

        let swinging = F::one() + (mass * movement).sqrt();
        let traction = movement / two;
        let speed = (*convergence * (F::one() + traction).ln()) / swinging;

        // Updating convergence
        *convergence = (((last_delta_x - delta_x).powi(2) + (last_delta_y - delta_y).powi(2))
            / swinging)
            .sqrt()
            .min(F::one());

        // Updating node position
        let new_x = x + delta_x * (speed / settings.slow_down);
        let new_y = y + delta_y * (speed / settings.slow_down);

        energy += (x - new_x).abs() + (y - new_y).abs();

        nodes[o3] = new_x;
        nodes[o3 + 1] = new_y;
    }

    energy
}
