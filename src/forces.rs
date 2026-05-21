use crate::settings::FA2Settings;
use crate::traits::Float;

#[allow(clippy::too_many_arguments)]
pub fn apply_forces<F: Float>(
    settings: &FA2Settings<F>,
    xs: &mut [F],
    ys: &mut [F],
    ms: &[F],
    delta_xs: &[F],
    delta_ys: &[F],
    old_delta_xs: &[F],
    old_delta_ys: &[F],
    convergences: &mut [F],
) -> F {
    let mut total_movement = F::zero();
    let two = F::from(2.0).unwrap();

    for (i, convergence) in convergences.iter_mut().enumerate() {
        let x = xs[i];
        let y = ys[i];
        let mass = ms[i];

        let delta_x = delta_xs[i];
        let delta_y = delta_ys[i];

        let old_delta_x = old_delta_xs[i];
        let old_delta_y = old_delta_ys[i];

        let mut swinging =
            mass * ((old_delta_x - delta_x).powi(2) + (old_delta_y - delta_y).powi(2)).sqrt();
        swinging = F::one() + swinging.sqrt();

        let traction =
            ((old_delta_x + delta_x).powi(2) + (old_delta_y + delta_y).powi(2)).sqrt() / two;

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

        xs[i] = new_x;
        ys[i] = new_y;
    }

    total_movement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_forces() {
        let settings = FA2Settings::<f32>::default();

        let mut xs = [1.0, 2.0];
        let mut ys = [3.0, -5.0];
        let ms = [1.0, 1.5];
        let delta_xs = [1.0, -4.0];
        let delta_ys = [2.0, -5.0];
        let mut convergences = [1.0, 1.0];

        let total_movement = apply_forces(
            &settings,
            &mut xs,
            &mut ys,
            &ms,
            &delta_xs,
            &delta_ys,
            &[0.0, 0.0],
            &[0.0, 0.0],
            &mut convergences,
        );

        assert_eq!(total_movement, 4.0539246);
        assert_eq!(xs, [1.3007548, 0.599262]);
        assert_eq!(ys, [3.6015096, -6.750922]);
        assert_eq!(convergences, [0.776293, 1.0]);
    }
}
