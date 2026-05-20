use crate::settings::FA2Settings;
use crate::traits::Float;

pub fn apply_forces<F: Float>(
    settings: &FA2Settings<F>,
    nodes: &mut [F],
    deltas: &[F],
    last_deltas: &[F],
    convergences: &mut [F],
) -> F {
    let mut total_movement = F::zero();
    let two = F::from(2.0).unwrap();

    for (o1, convergence) in convergences.iter_mut().enumerate() {
        let o2 = o1 * 2;
        let o3 = o1 * 3;

        let x = nodes[o3];
        let y = nodes[o3 + 1];
        let mass = nodes[o3 + 2];

        let delta_x = deltas[o2];
        let delta_y = deltas[o2 + 1];

        let last_delta_x = last_deltas[o2];
        let last_delta_y = last_deltas[o2 + 1];

        let mut swinging =
            mass * ((last_delta_x - delta_x).powi(2) + (last_delta_y - delta_y).powi(2)).sqrt();
        swinging = F::one() + swinging.sqrt();

        let traction =
            ((last_delta_x + delta_x).powi(2) + (last_delta_y + delta_y).powi(2)).sqrt() / two;

        let mut speed = *convergence * traction.ln_1p() / swinging;

        // Updating convergence
        *convergence = (speed * ((delta_x.powi(2) + delta_y.powi(2)) / swinging))
            .sqrt()
            .min(F::one());

        speed /= settings.slow_down;

        // Updating node position
        let new_x = x + delta_x * speed;
        let new_y = y + delta_y * speed;

        total_movement += (x - new_x).abs() + (y - new_y).abs();

        nodes[o3] = new_x;
        nodes[o3 + 1] = new_y;
    }

    total_movement
}
