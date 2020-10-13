mod event;
mod lineage;

pub mod settings;

use event::SimulationEvent;
use lineage::{LineageReference, SimulationLineages};

use crate::{landscape::Landscape, rng};

use settings::SimulationSettings;

pub struct Simulation(std::marker::PhantomData<()>);

impl Simulation {
    #[must_use]
    pub fn simulate<L: Landscape, R: rng::Rng>(
        settings: &SimulationSettings<L>,
        rng: &mut R,
    ) -> usize {
        let mut lineages = SimulationLineages::new(settings.landscape());

        // TODO: We should not print in library code.
        println!(
            "Starting the simulation with {} lineages ...",
            lineages.number_active_lineages()
        );

        let mut time: f64 = 0.0;
        let mut steps: usize = 0;

        let mut biodiversity: usize = 0;

        while let Some(chosen_active_lineage_reference) =
            lineages.pop_random_active_lineage_reference(rng)
        {
            if lineages.number_active_lineages() == 0 {
                biodiversity += 1;

                break;
            }

            time += Self::sample_delta_time(&lineages, rng);
            steps += 1;

            if let SimulationEvent::Speciation = Self::choose_and_perform_event_for_active_lineage(
                chosen_active_lineage_reference,
                settings,
                &mut lineages,
                rng,
            ) {
                biodiversity += 1;
            }
        }

        // TODO: We should not print in library code.
        println!("{} generations were simulated in {} steps.", time, steps);

        biodiversity
    }

    #[must_use]
    fn sample_delta_time(lineages: &SimulationLineages, rng: &mut impl rng::Rng) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * (lineages.number_active_lineages() + 1) as f64;

        rng.sample_exponential(lambda)
    }

    #[must_use]
    fn choose_and_perform_event_for_active_lineage<L: Landscape>(
        lineage_reference: LineageReference,
        settings: &SimulationSettings<L>,
        lineages: &mut SimulationLineages,
        rng: &mut impl rng::Rng,
    ) -> SimulationEvent {
        if rng.sample_event(settings.speciation_probability_per_generation()) {
            return SimulationEvent::Speciation;
        }

        let lineage = &lineages[lineage_reference];

        let dispersal_origin = lineage.location().clone();
        let dispersal_target = settings
            .landscape()
            .sample_dispersal_from_location(&dispersal_origin, rng);

        let optional_coalescence = lineages.sample_optional_coalescence_at_location(
            &dispersal_target,
            settings
                .landscape()
                .get_habitat_at_location(&dispersal_target),
            rng,
        );

        match optional_coalescence {
            None => {
                let event = if dispersal_origin == dispersal_target {
                    SimulationEvent::SelfDispersalNoCoalescence
                } else {
                    SimulationEvent::DispersalNoCoalescence
                };

                lineages
                    .push_active_lineage_reference_at_location(lineage_reference, dispersal_target);

                event
            }
            Some(_parent_lineage_reference) => {
                if dispersal_origin == dispersal_target {
                    SimulationEvent::SelfDispersalWithCoalescence
                } else {
                    SimulationEvent::DispersalWithCoalescence
                }
            }
        }
    }
}
