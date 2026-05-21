use crate::settings::FA2Settings;
use crate::traits::Float;

const SUBDIVISION_ATTEMPTS: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum Quadrant {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

impl Quadrant {
    #[inline(always)]
    fn offset(self, index: usize) -> usize {
        match self {
            Self::TopLeft => index,
            Self::BottomLeft => index + 1,
            Self::TopRight => index + 2,
            Self::BottomRight => index + 3,
        }
    }
}

#[derive(Debug, PartialEq)]
struct BarnesHutRegion<F: Float> {
    node: Option<usize>,
    center_x: F,
    center_y: F,
    size: F,
    next_sibling: Option<usize>,
    first_child: Option<usize>,
    mass: F,
    mass_center_x: F,
    mass_center_y: F,
}

impl<F: Float> BarnesHutRegion<F> {
    fn new_root(min_x: F, max_x: F, min_y: F, max_y: F) -> Self {
        let two = F::from(2.0).unwrap();

        Self {
            node: None,
            center_x: (min_x + max_x) / two,
            center_y: (min_y + max_y) / two,
            size: (max_x - min_x).max(max_y - min_y),
            next_sibling: None,
            first_child: None,
            mass: F::zero(),
            mass_center_x: F::zero(),
            mass_center_y: F::zero(),
        }
    }

    fn split(
        &self,
        from_index: usize,
        parent_sibling: Option<usize>,
        quadrant: Quadrant,
        size: F,
    ) -> Self {
        let (next_sibling, center_x, center_y) = match quadrant {
            Quadrant::TopLeft => (
                Some(from_index + 1),
                self.center_x - size,
                self.center_y - size,
            ),
            Quadrant::BottomLeft => (
                Some(from_index + 2),
                self.center_x - size,
                self.center_y + size,
            ),
            Quadrant::TopRight => (
                Some(from_index + 3),
                self.center_x + size,
                self.center_y - size,
            ),
            Quadrant::BottomRight => (parent_sibling, self.center_x + size, self.center_y + size),
        };

        Self {
            node: None,
            center_x,
            center_y,
            size,
            next_sibling,
            first_child: None,
            mass: F::zero(),
            mass_center_x: F::zero(),
            mass_center_y: F::zero(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BarnesHutTree<F: Float> {
    regions: Vec<BarnesHutRegion<F>>,
}

impl<F: Float> BarnesHutTree<F> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            regions: Vec::with_capacity(capacity),
        }
    }

    pub fn reset_with_extent(&mut self, extent: (F, F, F, F)) {
        self.regions.clear();

        let (mut min_x, mut max_x, mut min_y, mut max_y) = extent;

        // Squarifying bounds
        let dx = max_x - min_x;
        let dy = max_y - min_y;

        let two = F::from(2.0).unwrap();

        if dx > dy {
            min_y -= (dx - dy) / two;
            max_y = min_y + dx;
        } else {
            min_x -= (dy - dx) / two;
            max_x = min_x + dy;
        }

        self.regions = vec![BarnesHutRegion::new_root(min_x, max_x, min_y, max_y)];
    }

    pub fn read(&mut self, xs: &[F], ys: &[F], ms: &[F]) {
        let two = F::from(2.0).unwrap();

        let mut l: usize = 1;

        for n in 0..xs.len() {
            let mut region_index: usize = 0;
            let mut subdivision_attempts = SUBDIVISION_ATTEMPTS;

            let x = xs[n];
            let y = ys[n];
            let m = ms[n];

            loop {
                if let Some(first_child_index) = self.regions[region_index].first_child {
                    // There are sub-regions, we iterate to delve until we find a leaf
                    let current_region = &mut self.regions[region_index];

                    // Finding the quadrant of n
                    let quadrant = if x < current_region.center_x {
                        if y < current_region.center_y {
                            Quadrant::TopLeft
                        } else {
                            Quadrant::BottomLeft
                        }
                    } else if y < current_region.center_y {
                        Quadrant::TopRight
                    } else {
                        Quadrant::BottomRight
                    };

                    // Update mass
                    current_region.mass_center_x =
                        (current_region.mass_center_x * current_region.mass + x * m)
                            / (current_region.mass + m);

                    current_region.mass_center_y =
                        (current_region.mass_center_y * current_region.mass + y * m)
                            / (current_region.mass + m);

                    current_region.mass += m;

                    // Continue on relevant quadrant
                    region_index = quadrant.offset(first_child_index);

                    continue;
                }
                // There are no sub-regions, we are in a "leaf"
                else if let Some(region_node) = self.regions[region_index].node {
                    // There is a node in this region so we will need
                    // to create sub-regions
                    let current_region = &mut self.regions[region_index];
                    current_region.first_child = Some(l);

                    let half_size = current_region.size / two;

                    let mut top_left = current_region.split(
                        l,
                        current_region.next_sibling,
                        Quadrant::TopLeft,
                        half_size,
                    );

                    let mut bottom_left = current_region.split(
                        l,
                        current_region.next_sibling,
                        Quadrant::BottomLeft,
                        half_size,
                    );

                    let mut top_right = current_region.split(
                        l,
                        current_region.next_sibling,
                        Quadrant::TopRight,
                        half_size,
                    );

                    let mut bottom_right = current_region.split(
                        l,
                        current_region.next_sibling,
                        Quadrant::BottomRight,
                        half_size,
                    );

                    // Now we need to be able to put the two nodes in
                    // different sub-regions

                    // Finding old node's quadrant
                    let old_node_x = xs[region_node];
                    let old_node_y = ys[region_node];
                    let old_node_mass = ms[region_node];

                    let old_node_quadrant = if old_node_x < current_region.center_x {
                        if old_node_y < current_region.center_y {
                            top_left.node = Some(region_node);
                            Quadrant::TopLeft
                        } else {
                            bottom_left.node = Some(region_node);
                            Quadrant::BottomLeft
                        }
                    } else if old_node_y < current_region.center_y {
                        top_right.node = Some(region_node);
                        Quadrant::TopRight
                    } else {
                        bottom_right.node = Some(region_node);
                        Quadrant::BottomRight
                    };

                    current_region.mass = old_node_mass;
                    current_region.mass_center_x = old_node_x;
                    current_region.mass_center_y = old_node_y;

                    current_region.node = None;

                    // Finding the quadrant of n
                    let new_node_quadrant = if x < current_region.center_x {
                        if y < current_region.center_y {
                            Quadrant::TopLeft
                        } else {
                            Quadrant::BottomLeft
                        }
                    } else if y < current_region.center_y {
                        Quadrant::TopRight
                    } else {
                        Quadrant::BottomRight
                    };

                    // Pushing regions
                    self.regions.push(top_left);
                    self.regions.push(bottom_left);
                    self.regions.push(top_right);
                    self.regions.push(bottom_right);

                    l += 4;

                    if old_node_quadrant == new_node_quadrant {
                        // Both nodes fell in the same quadrant
                        subdivision_attempts -= 1;

                        if subdivision_attempts > 0 {
                            region_index = old_node_quadrant.offset(l - 4);
                            continue;
                        } else {
                            // We are out of precision here, we break anyway
                            break;
                        }
                    }

                    // Quadrants are different
                    self.regions[new_node_quadrant.offset(l - 4)].node = Some(n);
                    break;
                } else {
                    // There is no node in this region, we add it
                    self.regions[region_index].node = Some(n);
                    break;
                }
            }
        }
    }

    pub fn apply_repulsion(
        &self,
        settings: &FA2Settings<F>,
        xs: &[F],
        ys: &[F],
        ms: &[F],
        out_xs: &mut [F],
        out_ys: &mut [F],
    ) {
        let coefficient = settings.scaling_ratio;
        let theta_squared = settings.unwrap_barnes_hut_theta().powi(2);
        let four = F::from(4.0).unwrap();

        for n in 0..xs.len() {
            let mut current_region = &self.regions[0];

            let x = xs[n];
            let y = ys[n];
            let m = ms[n];

            loop {
                // There are sub-regions
                if let Some(first_child_index) = current_region.first_child {
                    let x_dist = x - current_region.mass_center_x;
                    let y_dist = y - current_region.mass_center_y;

                    let distance = x_dist * x_dist + y_dist * y_dist;

                    let size = current_region.size;

                    if (four * size * size) / distance < theta_squared {
                        // We treat the region as a single body for repulsion
                        if distance > F::zero() {
                            let factor = (coefficient * m * current_region.mass) / distance;

                            out_xs[n] += x_dist * factor;
                            out_ys[n] += y_dist * factor;
                        }

                        // Moving to next sibling
                        if let Some(next_sibling_index) = current_region.next_sibling {
                            current_region = &self.regions[next_sibling_index];
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        // This region is too close, we delve
                        current_region = &self.regions[first_child_index];
                        continue;
                    }
                }
                // There are no sub-regions
                else {
                    if let Some(region_node) = current_region.node {
                        if region_node != n {
                            let region_node_x = xs[region_node];
                            let region_node_y = ys[region_node];
                            let region_node_mass = ms[region_node];

                            let x_dist = x - region_node_x;
                            let y_dist = y - region_node_y;

                            let distance = x_dist * x_dist + y_dist * y_dist;

                            if distance > F::zero() {
                                let factor = (coefficient * m * region_node_mass) / distance;

                                out_xs[n] += x_dist * factor;
                                out_ys[n] += y_dist * factor;
                            }
                        }
                    }

                    // Moving to next sibling
                    if let Some(next_sibling_index) = current_region.next_sibling {
                        current_region = &self.regions[next_sibling_index];
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::FA2Data;

    use super::*;

    impl<F: Float> BarnesHutTree<F> {
        fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
            self.regions.iter().flat_map(|region| region.node)
        }
    }

    #[test]
    fn test_construction() {
        let mut data = FA2Data::new();
        data.add_node(1.0, 9.0);
        data.add_node(-1.0, 8.0);
        data.add_node(4.0, 1.0);
        data.add_node(9.0, 10.0);
        data.add_node(3.0, 0.0);

        let extent = data.positions_extent().unwrap();

        let mut tree = BarnesHutTree::with_capacity(5);
        tree.reset_with_extent(extent);
        tree.read(&data.xs, &data.ys, &data.ms);

        assert_eq!(tree.regions.len(), 13);
        assert_eq!(tree.nodes().collect::<Vec<_>>(), [4, 2, 3, 0]);
    }
}
