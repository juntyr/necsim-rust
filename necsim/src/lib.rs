#![deny(clippy::pedantic)]

use std::cmp::Ordering;

use rand::rngs::StdRng;

use array2d::Array2D;

trait RngCore {
    fn from_seed(seed: u64) -> Self;

    fn sample_uniform(&mut self) -> f64;
}

trait Rng: RngCore {
    fn sample_index(&mut self, length: usize) -> usize {
        // attributes on expressions are experimental
        // note: see issue #15701 <https://github.com/rust-lang/rust/issues/15701> for more information
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = (self.sample_uniform() * (length as f64)).floor() as usize;
        index
    }

    fn sample_exponential(&mut self, lambda: f64) -> f64 {
        -self.sample_uniform().ln() / lambda
    }

    fn sample_event(&mut self, probability: f64) -> bool {
        self.sample_uniform() < probability
    }
}

impl<T: RngCore> Rng for T {}

impl RngCore for StdRng {
    fn from_seed(seed: u64) -> Self {
        use rand::SeedableRng;

        StdRng::seed_from_u64(seed)
    }

    fn sample_uniform(&mut self) -> f64 {
        use rand::Rng;

        self.gen_range(0.0_f64, 1.0_f64)
    }
}

struct Simulation<R: RngCore> {
    lineages: Vec<Lineage>,
    active_lineages: Vec<usize>,
    time: f64,
    speciation_probability_per_generation: f64,
    biodiversity: usize,
    landscape: Landscape,
    rng: R,
}

#[must_use]
pub fn simulate(
    speciation_probability_per_generation: f64,
    landscape: Landscape,
    seed: u64,
) -> usize {
    let mut simulation: Simulation<StdRng> =
        Simulation::new(speciation_probability_per_generation, landscape, seed);

    while simulation.active_lineages.len() > 1 {
        simulation.time += simulation.sample_delta_t();

        let chosen_lineage = simulation.choose_active_lineage();

        if let Event::Speciation =
            simulation.choose_and_perform_event_for_active_lineage(chosen_lineage)
        {
            simulation.biodiversity += 1;
        }
    }

    simulation.biodiversity + 1
}

impl<R: Rng> Simulation<R> {
    fn new(
        speciation_probability_per_generation: f64,
        landscape: Landscape,
        seed: u64,
    ) -> Simulation<R> {
        Simulation {
            lineages: Vec::new(),
            active_lineages: Vec::new(),
            time: 0.0_f64,
            speciation_probability_per_generation,
            biodiversity: 0,
            landscape,
            rng: R::from_seed(seed),
        }
    }

    fn sample_delta_t(&mut self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * (self.active_lineages.len() as f64);

        self.rng.sample_exponential(lambda)
    }

    fn choose_active_lineage(&mut self) -> usize {
        self.rng.sample_index(self.active_lineages.len())
    }

    fn drop_active_lineage(&mut self, active_lineage: usize) {
        let last_active_lineage = self.active_lineages.pop().unwrap();

        if active_lineage < self.active_lineages.len() {
            self.active_lineages[active_lineage] = last_active_lineage;
        }
    }

    fn choose_and_perform_event_for_active_lineage(&mut self, active_lineage: usize) -> Event {
        let lineage_reference = self.active_lineages[active_lineage];

        self.landscape
            .remove_lineage(&mut self.lineages, lineage_reference);

        if self
            .rng
            .sample_event(self.speciation_probability_per_generation)
        {
            self.drop_active_lineage(active_lineage);

            return Event::Speciation;
        }

        let lineage = &mut self.lineages[lineage_reference];

        let dispersal_origin = lineage.pos.clone();
        let dispersal_target = self
            .landscape
            .sample_dispersal_from_position(&mut self.rng, &dispersal_origin);

        let optional_coalescence = self
            .landscape
            .sample_optional_coalescence_lineage_at_position(&mut self.rng, &dispersal_target);

        match optional_coalescence {
            None => {
                lineage.pos = dispersal_target.clone();

                self.landscape
                    .add_lineage(&mut self.lineages, lineage_reference);

                if dispersal_origin == dispersal_target {
                    Event::SelfDispersalNoCoalescence
                } else {
                    Event::DispersalNoCoalescence
                }
            }
            Some(_parent) => {
                self.drop_active_lineage(active_lineage);

                if dispersal_origin == dispersal_target {
                    Event::SelfDispersalWithCoalescence
                } else {
                    Event::DispersalWithCoalescence
                }
            }
        }
    }
}

enum Event {
    Speciation,
    SelfDispersalWithCoalescence,
    SelfDispersalNoCoalescence,
    DispersalWithCoalescence,
    DispersalNoCoalescence,
}

pub struct Landscape {
    habitat: Array2D<u32>,
    cumulative_dispersal: Vec<f64>,
    lineages: Array2D<Vec<usize>>,
}

pub struct InconsistentDispersalMapSize;

impl Landscape {
    /// Creates a new `Landscape` from the `habitat` and `dispersal` map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    pub fn new(
        habitat: Array2D<u32>,
        dispersal: &Array2D<f64>,
    ) -> Result<Landscape, InconsistentDispersalMapSize> {
        if dispersal.num_rows() != habitat.num_elements()
            || dispersal.num_columns() != habitat.num_elements()
        {
            return Err(InconsistentDispersalMapSize);
        }

        let mut cumulative_dispersal = vec![0.0_f64; dispersal.num_elements()];

        for row_index in 0..dispersal.num_rows() {
            let sum: f64 = dispersal.row_iter(row_index).sum();
            let mut acc = 0.0_f64;

            for col_index in 0..dispersal.num_columns() {
                acc += dispersal[(row_index, col_index)];

                cumulative_dispersal[row_index * dispersal.row_len() + col_index] = acc / sum;
            }
        }

        Ok(Landscape {
            lineages: Array2D::filled_with(Vec::new(), habitat.num_rows(), habitat.num_columns()),
            habitat,
            cumulative_dispersal,
        })
    }

    fn sample_dispersal_from_position(&self, rng: &mut impl Rng, pos: &Position) -> Position {
        let sample = rng.sample_uniform();

        let pos_index = (pos.y as usize) * self.habitat.row_len() + (pos.x as usize);

        let cumulative_dispersals_at_pos = &self.cumulative_dispersal[pos_index
            * self.habitat.num_elements()
            ..(pos_index + 1) * self.habitat.num_elements()];

        let dispersal_target_index = match cumulative_dispersals_at_pos
            .binary_search_by(|v| v.partial_cmp(&sample).unwrap_or(Ordering::Equal))
        {
            Ok(index) | Err(index) => index,
        };

        #[allow(clippy::cast_possible_truncation)]
        Position {
            y: (dispersal_target_index / self.habitat.row_len()) as u32,
            x: (dispersal_target_index % self.habitat.row_len()) as u32,
        }
    }

    fn add_lineage(&mut self, lineages: &mut Vec<Lineage>, lineage_reference: usize) {
        let lineage = &lineages[lineage_reference];
        let position = (lineage.pos.y as usize, lineage.pos.x as usize);

        let lineages_at_pos = &mut self.lineages[position];

        assert!(lineages_at_pos.len() < (self.habitat[position] as usize));

        lineages_at_pos.push(lineage_reference);
        lineages[lineage_reference].index = lineages_at_pos.len();
    }

    fn remove_lineage(&mut self, lineages: &mut Vec<Lineage>, lineage_reference: usize) {
        let lineage = &lineages[lineage_reference];
        let position = (lineage.pos.y as usize, lineage.pos.x as usize);

        let lineages_at_pos = &mut self.lineages[position];

        let last_lineage_at_pos = lineages_at_pos.pop().unwrap();

        if lineage.index < lineages_at_pos.len() {
            lineages_at_pos[lineage.index] = last_lineage_at_pos;
        }
    }

    fn get_number_active_lineages_at_position(&self, pos: &Position) -> usize {
        self.lineages[(pos.y as usize, pos.x as usize)].len()
    }

    fn get_habitat_at_position(&self, pos: &Position) -> usize {
        self.habitat[(pos.y as usize, pos.x as usize)] as usize
    }

    fn sample_optional_coalescence_lineage_at_position(
        &self,
        rng: &mut impl Rng,
        pos: &Position,
    ) -> Option<usize> {
        let habitat = self.get_habitat_at_position(pos);
        let population = self.get_number_active_lineages_at_position(pos);

        let chosen_coalescence = rng.sample_index(habitat);

        if chosen_coalescence >= population {
            return None;
        }

        Some(self.lineages[(pos.y as usize, pos.x as usize)][chosen_coalescence])
    }
}

struct Lineage {
    pos: Position,
    index: usize,
}

#[derive(Eq, PartialEq, Clone)]
struct Position {
    x: u32,
    y: u32,
}
