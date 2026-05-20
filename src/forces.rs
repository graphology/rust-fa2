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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_forces() {
        let settings = FA2Settings::<f32>::default();

        let mut nodes = [1.0, 3.0, 1.0, 2.0, -5.0, 1.5];
        let deltas = [1.0, 2.0, -4.0, -5.0];
        let last_deltas = [0.0, 0.0, 0.0, 0.0];
        let mut convergences = [1.0, 1.0];

        let total_movement = apply_forces(
            &settings,
            &mut nodes,
            &deltas,
            &last_deltas,
            &mut convergences,
        );

        assert_eq!(total_movement, 4.0539246);
        assert_eq!(nodes, [1.3007548, 3.6015096, 1.0, 0.599262, -6.750922, 1.5]);
        assert_eq!(convergences, [0.776293, 1.0]);
    }
}
