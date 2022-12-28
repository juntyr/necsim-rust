use necsim_core_maths::MathsCore;

use super::{
    utils,
    vector::{vec2::Vec2, VecMethods},
    NoiseEvaluator, PermTable,
};

const STRETCH: f64 = -0.211_324_865_405_187; // (1 / sqrt(2 + 1) - 1) / 2
const SQUISH: f64 = 0.366_025_403_784_439; // (sqrt(2 + 1) - 1) / 2

const NORMALIZING_SCALAR: f64 = 47.0;

pub enum OpenSimplexNoise2D {}

impl NoiseEvaluator<Vec2<f64>> for OpenSimplexNoise2D {
    const SQUISH_POINT: Vec2<f64> = Vec2::new(SQUISH, SQUISH);
    const STRETCH_POINT: Vec2<f64> = Vec2::new(STRETCH, STRETCH);

    fn extrapolate<M: MathsCore>(
        grid: Vec2<f64>,
        delta: Vec2<f64>,
        perm: &PermTable,
        wrap: f64,
    ) -> f64 {
        // Wrap the grid to put in the range [0; wrap), then snap to grid points
        let grid = grid.map(|i| i - M::floor(i / wrap) * wrap).map(M::floor);

        let idx = Self::get_grad_table_index(grid, perm);

        // Calculate the gradient directly to avoid a GRAD_TABLE lookup
        // const GRAD_TABLE: [Vec2<f64>; 8] = [
        //     Vec2::new(5.0, 2.0),
        //     Vec2::new(2.0, 5.0),
        //     Vec2::new(-5.0, 2.0),
        //     Vec2::new(-2.0, 5.0),
        //     Vec2::new(5.0, -2.0),
        //     Vec2::new(2.0, -5.0),
        //     Vec2::new(-5.0, -2.0),
        //     Vec2::new(-2.0, -5.0),
        // ];
        let mut point = if (idx & 1) == 0 {
            Vec2::new(5.0, 2.0)
        } else {
            Vec2::new(2.0, 5.0)
        };
        if (idx & 2) != 0 {
            point.x *= -1.0;
        }
        if (idx & 4) != 0 {
            point.y *= -1.0;
        }

        // let point = GRAD_TABLE[Self::get_grad_table_index(grid, perm)];
        point.x * delta.x + point.y * delta.y
    }

    fn eval<M: MathsCore>(input: Vec2<f64>, perm: &PermTable, wrap: f64) -> f64 {
        // Pre-squish the input to allow wrapping in extrapolate
        let input = input + (Self::SQUISH_POINT * input.sum());

        let stretch: Vec2<f64> = input + (Self::STRETCH_POINT * input.sum());
        let grid = stretch.map(M::floor);

        let squashed: Vec2<f64> = grid + (Self::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise2D::get_value::<M>(grid, origin, ins, perm, wrap)
    }
}

impl OpenSimplexNoise2D {
    #[allow(clippy::inline_always)]
    #[cfg_attr(target_os = "cuda", inline(always))]
    fn get_value<M: MathsCore>(
        grid: Vec2<f64>,
        origin: Vec2<f64>,
        ins: Vec2<f64>,
        perm: &PermTable,
        wrap: f64,
    ) -> f64 {
        let contribute = |x, y| -> f64 {
            utils::contribute::<OpenSimplexNoise2D, Vec2<f64>, M>(
                Vec2::new(x, y),
                origin,
                grid,
                perm,
                wrap,
            )
        };

        let value = contribute(1.0, 0.0)
            + contribute(0.0, 1.0)
            + Self::evaluate_inside_triangle(ins, contribute);

        value / NORMALIZING_SCALAR
    }

    fn evaluate_inside_triangle(ins: Vec2<f64>, contribute: impl Fn(f64, f64) -> f64) -> f64 {
        let in_sum = ins.sum();
        let factor_point = match in_sum {
            x if x <= 1.0 => Vec2::new(0.0, 0.0),
            _ => Vec2::new(1.0, 1.0),
        };
        Self::evaluate_inside_triangle_at(factor_point, in_sum, ins, contribute)
    }

    fn evaluate_inside_triangle_at(
        factor_point: Vec2<f64>,
        in_sum: f64,
        ins: Vec2<f64>,
        contribute: impl Fn(f64, f64) -> f64,
    ) -> f64 {
        let z_ins = 1.0 + factor_point.x - in_sum;
        let point = if z_ins > ins.x || z_ins > ins.y {
            // (0, 0) is one of the closest two triangular vertices
            if ins.x > ins.y {
                Vec2::new(1.0 + factor_point.x, -1.0 + factor_point.y)
            } else {
                Vec2::new(-1.0 + factor_point.x, 1.0 + factor_point.y)
            }
        } else {
            // (1, 0) and (0, 1) are the closest two vertices.
            Vec2::new(1.0 - factor_point.x, 1.0 - factor_point.y)
        };

        contribute(0.0 + factor_point.x, 0.0 + factor_point.y) + contribute(point.x, point.y)
    }

    fn get_grad_table_index(grid: Vec2<f64>, perm: &PermTable) -> usize {
        let index0 = ((perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF) as usize;
        ((perm[index0] & 0x0E) >> 1) as usize
    }
}
