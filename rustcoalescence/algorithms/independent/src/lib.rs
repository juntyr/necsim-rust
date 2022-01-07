#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate serde_derive_state;

use std::marker::PhantomData;

use arguments::{
    IndependentArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
    ProbabilisticParallelismMode,
};
use necsim_core::{
    cogs::{EmigrationExit, MathsCore, PrimeableRng},
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;
use necsim_core_maths::IntrinsicsMathsCore;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::{poisson::PoissonEventTimeSampler, EventTimeSampler},
            IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
        emigration_exit::{
            independent::{
                choice::{
                    always::AlwaysEmigrationChoice, probabilistic::ProbabilisticEmigrationChoice,
                },
                IndependentEmigrationExit,
            },
            never::NeverEmigrationExit,
        },
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
            resuming::ResumingOriginSampler, TrustedOriginSampler,
        },
        rng::wyhash::WyHash,
    },
    parallelisation::{self, Status},
};
use necsim_partitioning_core::LocalPartition;

mod arguments;

use rustcoalescence_algorithms::{Algorithm, AlgorithmParamters, AlgorithmResult, ContinueError};
use rustcoalescence_scenarios::Scenario;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum IndependentAlgorithm {}

impl AlgorithmParamters for IndependentAlgorithm {
    type Arguments = IndependentArguments;
    type Error = !;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<IntrinsicsMathsCore, WyHash<IntrinsicsMathsCore>>,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for IndependentAlgorithm
{
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<IntrinsicsMathsCore, O::Habitat>;
    type MathsCore = IntrinsicsMathsCore;
    type Rng = WyHash<IntrinsicsMathsCore>;

    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error> {
        struct GenesisInitialiser;

        impl<M: MathsCore, G: PrimeableRng<M>, O: Scenario<M, G>>
            IndependentLineageStoreSampleInitialiser<M, G, O, !> for GenesisInitialiser
        {
            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate>,
                X: EmigrationExit<
                    M,
                    O::Habitat,
                    G,
                    GlobalLineageReference,
                    IndependentLineageStore<M, O::Habitat>,
                >,
            >(
                self,
                origin_sampler: T,
                event_time_sampler: J,
            ) -> Result<
                (
                    IndependentLineageStore<M, O::Habitat>,
                    IndependentActiveLineageSampler<
                        M,
                        O::Habitat,
                        G,
                        X,
                        O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                        J,
                    >,
                    Vec<Lineage>,
                ),
                !,
            >
            where
                O::Habitat: 'h,
            {
                Ok(
                    IndependentActiveLineageSampler::init_with_store_and_lineages(
                        origin_sampler,
                        event_time_sampler,
                    ),
                )
            }
        }

        initialise_and_simulate(
            &args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            GenesisInitialiser,
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError::Sample` if initialising the resuming
    ///  simulation failed
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, ContinueError<Self::Error>> {
        struct ResumeInitialiser<L: ExactSizeIterator<Item = Lineage>> {
            lineages: L,
        }

        impl<
                L: ExactSizeIterator<Item = Lineage>,
                M: MathsCore,
                G: PrimeableRng<M>,
                O: Scenario<M, G>,
            > IndependentLineageStoreSampleInitialiser<M, G, O, ContinueError<!>>
            for ResumeInitialiser<L>
        {
            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate>,
                X: EmigrationExit<
                    M,
                    O::Habitat,
                    G,
                    GlobalLineageReference,
                    IndependentLineageStore<M, O::Habitat>,
                >,
            >(
                self,
                origin_sampler: T,
                event_time_sampler: J,
            ) -> Result<
                (
                    IndependentLineageStore<M, O::Habitat>,
                    IndependentActiveLineageSampler<
                        M,
                        O::Habitat,
                        G,
                        X,
                        O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                        J,
                    >,
                    Vec<Lineage>,
                ),
                ContinueError<!>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let (lineage_store, active_lineage_sampler, lineages, exceptional_lineages) =
                    IndependentActiveLineageSampler::resume_with_store_and_lineages(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                        event_time_sampler,
                        NonNegativeF64::zero(),
                    );

                if !exceptional_lineages.is_empty() {
                    return Err(ContinueError::Sample(exceptional_lineages));
                }

                Ok((lineage_store, active_lineage_sampler, lineages))
            }
        }

        initialise_and_simulate(
            &args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            ResumeInitialiser { lineages },
        )
    }
}

#[allow(clippy::too_many_lines)]
fn initialise_and_simulate<
    M: MathsCore,
    G: PrimeableRng<M>,
    O: Scenario<M, G>,
    R: Reporter,
    P: LocalPartition<R>,
    I: Iterator<Item = u64>,
    L: IndependentLineageStoreSampleInitialiser<M, G, O, Error>,
    Error,
>(
    args: &IndependentArguments,
    rng: G,
    scenario: O,
    pre_sampler: OriginPreSampler<M, I>,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    lineage_store_sampler_initialiser: L,
) -> Result<AlgorithmResult<M, G>, Error> {
    match args.parallelism_mode {
        ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
        | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { event_slice, .. })
        | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                decomposition_auxiliary,
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = IndependentCoalescenceSampler::default();
            let event_sampler = IndependentEventSampler::default();

            let (lineage_store, active_lineage_sampler, lineages) = match args.parallelism_mode {
                // Apply no lineage origin partitioning in the `Monolithic` mode
                ParallelismMode::Monolithic(..) => lineage_store_sampler_initialiser.init(
                    O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?,
                // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
                ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                    partition, ..
                }) => lineage_store_sampler_initialiser.init(
                    O::sample_habitat(
                        &habitat,
                        pre_sampler.partition(partition),
                        origin_sampler_auxiliary,
                    ),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?,
                // Apply lineage origin partitioning in the `IsolatedLandscape` mode
                ParallelismMode::IsolatedLandscape(IsolatedParallelismMode {
                    partition, ..
                }) => lineage_store_sampler_initialiser.init(
                    DecompositionOriginSampler::new(
                        O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                        &O::decompose(&habitat, partition, decomposition_auxiliary),
                    ),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?,
                _ => unsafe { std::hint::unreachable_unchecked() },
            };

            let emigration_exit = NeverEmigrationExit::default();
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<GlobalLineageReference>,
                lineage_store,
                dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                emigration_exit,
                event_sampler,
                active_lineage_sampler,
                rng,
                immigration_entry,
            }
            .build();

            let (status, time, steps, lineages) =
                parallelisation::independent::monolithic::simulate(
                    &mut simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    event_slice,
                    pause_before,
                    local_partition,
                );

            match status {
                Status::Done => Ok(AlgorithmResult::Done { time, steps }),
                Status::Paused => Ok(AlgorithmResult::Paused {
                    time,
                    steps,
                    lineages: lineages.into_iter().collect(),
                    rng: simulation.rng_mut().clone(),
                    marker: PhantomData,
                }),
            }
        },
        ParallelismMode::Individuals => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                _decomposition_auxiliary,
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = IndependentCoalescenceSampler::default();
            let event_sampler = IndependentEventSampler::default();

            let (lineage_store, active_lineage_sampler, lineages) =
                lineage_store_sampler_initialiser.init(
                    O::sample_habitat(
                        &habitat,
                        pre_sampler.partition(local_partition.get_partition()),
                        origin_sampler_auxiliary,
                    ),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?;

            let emigration_exit = NeverEmigrationExit::default();
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<GlobalLineageReference>,
                lineage_store,
                dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                emigration_exit,
                event_sampler,
                active_lineage_sampler,
                rng,
                immigration_entry,
            }
            .build();

            let (_status, time, steps, _lineages) =
                parallelisation::independent::individuals::simulate(
                    &mut simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                );

            // TODO: Adapt for parallel pausing
            Ok(AlgorithmResult::Done { time, steps })
        },
        ParallelismMode::Landscape => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                decomposition_auxiliary,
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = IndependentCoalescenceSampler::default();
            let event_sampler = IndependentEventSampler::default();

            let decomposition = O::decompose(
                &habitat,
                local_partition.get_partition(),
                decomposition_auxiliary,
            );

            let (lineage_store, active_lineage_sampler, lineages) =
                lineage_store_sampler_initialiser.init(
                    DecompositionOriginSampler::new(
                        O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                        &decomposition,
                    ),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?;

            let emigration_exit =
                IndependentEmigrationExit::new(decomposition, AlwaysEmigrationChoice::default());
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<GlobalLineageReference>,
                lineage_store,
                dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                emigration_exit,
                event_sampler,
                active_lineage_sampler,
                rng,
                immigration_entry,
            }
            .build();

            let (_status, time, steps, _lineages) =
                parallelisation::independent::landscape::simulate(
                    &mut simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                );

            // TODO: Adapt for parallel pausing
            Ok(AlgorithmResult::Done { time, steps })
        },
        ParallelismMode::Probabilistic(ProbabilisticParallelismMode {
            communication_probability,
        }) => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                decomposition_auxiliary,
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = IndependentCoalescenceSampler::default();
            let event_sampler = IndependentEventSampler::default();

            let decomposition = O::decompose(
                &habitat,
                local_partition.get_partition(),
                decomposition_auxiliary,
            );

            let (lineage_store, active_lineage_sampler, lineages) =
                lineage_store_sampler_initialiser.init(
                    DecompositionOriginSampler::new(
                        O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                        &decomposition,
                    ),
                    PoissonEventTimeSampler::new(args.delta_t),
                )?;

            let emigration_exit = IndependentEmigrationExit::new(
                decomposition,
                ProbabilisticEmigrationChoice::new(communication_probability),
            );
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<GlobalLineageReference>,
                lineage_store,
                dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                emigration_exit,
                event_sampler,
                active_lineage_sampler,
                rng,
                immigration_entry,
            }
            .build();

            let (_status, time, steps, _lineages) =
                parallelisation::independent::landscape::simulate(
                    &mut simulation,
                    lineages,
                    args.dedup_cache,
                    args.step_slice,
                    local_partition,
                );

            // TODO: Adapt for parallel pausing
            Ok(AlgorithmResult::Done { time, steps })
        },
    }
}

#[allow(clippy::type_complexity)]
trait IndependentLineageStoreSampleInitialiser<
    M: MathsCore,
    G: PrimeableRng<M>,
    O: Scenario<M, G>,
    Error,
>
{
    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate>,
        X: EmigrationExit<
            M,
            O::Habitat,
            G,
            GlobalLineageReference,
            IndependentLineageStore<M, O::Habitat>,
        >,
    >(
        self,
        origin_sampler: T,
        event_time_sampler: J,
    ) -> Result<
        (
            IndependentLineageStore<M, O::Habitat>,
            IndependentActiveLineageSampler<
                M,
                O::Habitat,
                G,
                X,
                O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
                O::TurnoverRate,
                O::SpeciationProbability,
                J,
            >,
            Vec<Lineage>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}
