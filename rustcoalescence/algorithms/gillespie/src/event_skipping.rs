use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, EmigrationExit, GloballyCoherentLineageStore, ImmigrationEntry,
        LineageReference, MathsCore, RngCore, SeparableDispersalSampler, SplittableRng,
    },
    event::DispersalEvent,
    lineage::{Lineage, LineageInteraction},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_core_maths::IntrinsicsMathsCore;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::{
            alias::location::LocationAliasActiveLineageSampler,
            resuming::{
                ExceptionalLineage, RestartFixUpActiveLineageSampler, SplitExceptionalLineages,
            },
        },
        coalescence_sampler::conditional::ConditionalCoalescenceSampler,
        dispersal_sampler::{
            in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
            trespassing::{
                uniform::UniformAntiTrespassingDispersalSampler, TrespassingDispersalSampler,
            },
        },
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::globally::gillespie::GillespieLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
            resuming::ResumingOriginSampler, TrustedOriginSampler,
        },
    },
    parallelisation::{self, Status},
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{
    Algorithm, AlgorithmParamters, AlgorithmResult, CoalescenceStrategy, ContinueError,
    OutOfDemeStrategy, OutOfHabitatStrategy, RestartFixUpStrategy,
};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct EventSkippingAlgorithm {}

impl AlgorithmParamters for EventSkippingAlgorithm {
    type Arguments = MonolithicArguments;
    type Error = !;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsMathsCore,
            Pcg<IntrinsicsMathsCore>,
            LineageReference = InMemoryLineageReference,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for EventSkippingAlgorithm
where
    O::LineageStore<GillespieLineageStore<IntrinsicsMathsCore, O::Habitat>>:
        GloballyCoherentLineageStore<IntrinsicsMathsCore, O::Habitat, InMemoryLineageReference>,
    O::DispersalSampler<
        InMemorySeparableAliasDispersalSampler<
            IntrinsicsMathsCore,
            O::Habitat,
            Pcg<IntrinsicsMathsCore>,
        >,
    >: SeparableDispersalSampler<IntrinsicsMathsCore, O::Habitat, Pcg<IntrinsicsMathsCore>>,
{
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<GillespieLineageStore<Self::MathsCore, O::Habitat>>;
    type MathsCore = IntrinsicsMathsCore;
    type Rng = Pcg<Self::MathsCore>;

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error> {
        struct GenesisInitialiser;

        impl<M: MathsCore, G: RngCore<M>, O: Scenario<M, G>>
            EventSkippingLineageStoreSampleInitialiser<M, G, O, !> for GenesisInitialiser
        where
            O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
                SeparableDispersalSampler<M, O::Habitat, G>,
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
            > = LocationAliasActiveLineageSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                O::TurnoverRate,
                O::SpeciationProbability,
                ConditionalGillespieEventSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    O::TurnoverRate,
                    O::SpeciationProbability,
                >,
                I,
            >;
            type DispersalSampler =
                O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
                >,
                coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                turnover_rate: &O::TurnoverRate,
                speciation_probability: &O::SpeciationProbability,
                _local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    ConditionalGillespieEventSampler<
                        M,
                        O::Habitat,
                        G,
                        R,
                        S,
                        X,
                        Self::DispersalSampler,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                    >,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                !,
            >
            where
                O::Habitat: 'h,
            {
                let event_sampler = ConditionalGillespieEventSampler::default();

                let (lineage_store, active_lineage_sampler) =
                    LocationAliasActiveLineageSampler::init_with_store(
                        origin_sampler,
                        &dispersal_sampler,
                        coalescence_sampler,
                        turnover_rate,
                        speciation_probability,
                        &event_sampler,
                    );

                Ok((
                    lineage_store,
                    dispersal_sampler,
                    event_sampler,
                    active_lineage_sampler,
                ))
            }
        }

        initialise_and_simulate(
            args,
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
    #[allow(clippy::too_many_lines)]
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, ContinueError<Self::Error>> {
        struct ResumeInitialiser<L: ExactSizeIterator<Item = Lineage>> {
            lineages: L,
            resume_after: Option<NonNegativeF64>,
        }

        impl<
                L: ExactSizeIterator<Item = Lineage>,
                M: MathsCore,
                G: RngCore<M>,
                O: Scenario<M, G>,
            > EventSkippingLineageStoreSampleInitialiser<M, G, O, ContinueError<!>>
            for ResumeInitialiser<L>
        where
            O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
                SeparableDispersalSampler<M, O::Habitat, G>,
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
            > = LocationAliasActiveLineageSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                O::TurnoverRate,
                O::SpeciationProbability,
                ConditionalGillespieEventSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    O::TurnoverRate,
                    O::SpeciationProbability,
                >,
                I,
            >;
            type DispersalSampler =
                O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
                >,
                coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                turnover_rate: &O::TurnoverRate,
                speciation_probability: &O::SpeciationProbability,
                _local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    ConditionalGillespieEventSampler<
                        M,
                        O::Habitat,
                        G,
                        R,
                        S,
                        X,
                        Self::DispersalSampler,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                    >,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                ContinueError<!>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let event_sampler = ConditionalGillespieEventSampler::default();

                let (lineage_store, active_lineage_sampler, exceptional_lineages) =
                    LocationAliasActiveLineageSampler::resume_with_store(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                        &dispersal_sampler,
                        coalescence_sampler,
                        turnover_rate,
                        speciation_probability,
                        &event_sampler,
                        self.resume_after.unwrap_or(NonNegativeF64::zero()),
                    );

                if !exceptional_lineages.is_empty() {
                    return Err(ContinueError::Sample(exceptional_lineages));
                }

                Ok((
                    lineage_store,
                    dispersal_sampler,
                    event_sampler,
                    active_lineage_sampler,
                ))
            }
        }

        initialise_and_simulate(
            args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            ResumeInitialiser {
                lineages,
                resume_after,
            },
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if fixing up the restarting
    ///  simulation (incl. running the algorithm) failed
    #[allow(clippy::too_many_lines)]
    fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        restart_at: PositiveF64,
        fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, ContinueError<Self::Error>> {
        struct FixUpInitialiser<L: ExactSizeIterator<Item = Lineage>> {
            lineages: L,
            restart_at: PositiveF64,
            fixup_strategy: RestartFixUpStrategy,
        }

        impl<
                L: ExactSizeIterator<Item = Lineage>,
                M: MathsCore,
                G: RngCore<M>,
                O: Scenario<M, G>,
            > EventSkippingLineageStoreSampleInitialiser<M, G, O, ContinueError<!>>
            for FixUpInitialiser<L>
        where
            O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
                SeparableDispersalSampler<M, O::Habitat, G>,
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
            > = RestartFixUpActiveLineageSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                O::TurnoverRate,
                O::SpeciationProbability,
                ConditionalGillespieEventSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    O::TurnoverRate,
                    O::SpeciationProbability,
                >,
                I,
                LocationAliasActiveLineageSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                    O::TurnoverRate,
                    O::SpeciationProbability,
                    ConditionalGillespieEventSampler<
                        M,
                        O::Habitat,
                        G,
                        R,
                        S,
                        X,
                        Self::DispersalSampler,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                    >,
                    I,
                >,
            >;
            type DispersalSampler = TrespassingDispersalSampler<
                M,
                O::Habitat,
                G,
                O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, G>,
            >;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: GloballyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
                >,
                coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
                turnover_rate: &O::TurnoverRate,
                speciation_probability: &O::SpeciationProbability,
                local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    ConditionalGillespieEventSampler<
                        M,
                        O::Habitat,
                        G,
                        R,
                        S,
                        X,
                        Self::DispersalSampler,
                        O::TurnoverRate,
                        O::SpeciationProbability,
                    >,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                ContinueError<!>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let event_sampler = ConditionalGillespieEventSampler::default();
                let dispersal_sampler = TrespassingDispersalSampler::new(
                    dispersal_sampler,
                    UniformAntiTrespassingDispersalSampler::default(),
                );

                let (lineage_store, active_lineage_sampler, exceptional_lineages) =
                    LocationAliasActiveLineageSampler::resume_with_store(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                        &dispersal_sampler,
                        coalescence_sampler,
                        turnover_rate,
                        speciation_probability,
                        &event_sampler,
                        self.restart_at.into(),
                    );

                let SplitExceptionalLineages {
                    mut coalescence,
                    out_of_deme,
                    out_of_habitat,
                } = ExceptionalLineage::split_vec(exceptional_lineages);

                let mut exceptional_lineages = Vec::new();
                let mut fixable_lineages = Vec::new();

                match self.fixup_strategy.coalescence {
                    CoalescenceStrategy::Abort => {
                        exceptional_lineages.extend(coalescence.into_iter().map(
                            |(child, parent)| ExceptionalLineage::Coalescence { child, parent },
                        ));
                    },
                    CoalescenceStrategy::Coalescence => {
                        coalescence.sort();

                        for (coalescing_lineage, parent) in coalescence {
                            local_partition.get_reporter().report_dispersal(
                                (&DispersalEvent {
                                    global_lineage_reference: coalescing_lineage.global_reference,
                                    prior_time: coalescing_lineage.last_event_time,
                                    event_time: self.restart_at,
                                    origin: coalescing_lineage.indexed_location.clone(),
                                    target: coalescing_lineage.indexed_location,
                                    interaction: LineageInteraction::Coalescence(parent),
                                })
                                    .into(),
                            );
                        }
                    },
                }

                match self.fixup_strategy.out_of_deme {
                    OutOfDemeStrategy::Abort => {
                        exceptional_lineages
                            .extend(out_of_deme.into_iter().map(ExceptionalLineage::OutOfDeme));
                    },
                    OutOfDemeStrategy::Dispersal => {
                        fixable_lineages.extend(out_of_deme.into_iter());
                    },
                }

                match self.fixup_strategy.out_of_habitat {
                    OutOfHabitatStrategy::Abort => {
                        exceptional_lineages.extend(
                            out_of_habitat
                                .into_iter()
                                .map(ExceptionalLineage::OutOfHabitat),
                        );
                    },
                    OutOfHabitatStrategy::UniformDispersal => {
                        fixable_lineages.extend(out_of_habitat.into_iter());
                    },
                }

                if !exceptional_lineages.is_empty() {
                    return Err(ContinueError::Sample(exceptional_lineages));
                }

                fixable_lineages.sort();

                let active_lineage_sampler = RestartFixUpActiveLineageSampler::new(
                    active_lineage_sampler,
                    fixable_lineages,
                    self.restart_at,
                );

                Ok((
                    lineage_store,
                    dispersal_sampler,
                    event_sampler,
                    active_lineage_sampler,
                ))
            }
        }

        initialise_and_simulate(
            args,
            rng,
            scenario,
            pre_sampler,
            Some(PositiveF64::max_after(restart_at.into(), restart_at.into()).into()),
            local_partition,
            FixUpInitialiser {
                lineages,
                restart_at,
                fixup_strategy,
            },
        )
    }
}

#[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
fn initialise_and_simulate<
    M: MathsCore,
    G: SplittableRng<M>,
    O: Scenario<M, G, LineageReference = InMemoryLineageReference>,
    R: Reporter,
    P: LocalPartition<R>,
    I: Iterator<Item = u64>,
    L: EventSkippingLineageStoreSampleInitialiser<M, G, O, Error>,
    Error,
>(
    args: MonolithicArguments,
    rng: G,
    scenario: O,
    pre_sampler: OriginPreSampler<M, I>,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    lineage_store_sampler_initialiser: L,
) -> Result<AlgorithmResult<M, G>, Error>
where
    O::LineageStore<GillespieLineageStore<M, O::Habitat>>:
        GloballyCoherentLineageStore<M, O::Habitat, InMemoryLineageReference>,
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    match args.parallelism_mode {
        ParallelismMode::Monolithic => {
            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                _decomposition_auxiliary,
            ) = scenario.build::<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = ConditionalCoalescenceSampler::default();

            let (lineage_store, dispersal_sampler, event_sampler, active_lineage_sampler): (
                O::LineageStore<GillespieLineageStore<M, O::Habitat>>,
                _,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                dispersal_sampler,
                &coalescence_sampler,
                &turnover_rate,
                &speciation_probability,
                local_partition,
            )?;

            let emigration_exit = NeverEmigrationExit::default();
            let immigration_entry = NeverImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<InMemoryLineageReference>,
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

            let (status, time, steps) = parallelisation::monolithic::monolithic::simulate(
                &mut simulation,
                pause_before,
                local_partition,
            );

            match status {
                Status::Done => Ok(AlgorithmResult::Done { time, steps }),
                Status::Paused => Ok(AlgorithmResult::Paused {
                    time,
                    steps,
                    lineages: simulation
                        .active_lineage_sampler()
                        .iter_active_lineages_ordered(
                            simulation.habitat(),
                            simulation.lineage_store(),
                        )
                        .cloned()
                        .collect(),
                    rng: simulation.rng_mut().clone(),
                    marker: PhantomData,
                }),
            }
        },
        non_monolithic_parallelism_mode => {
            let rng = rng.split_to_stream(u64::from(local_partition.get_partition().rank()));

            let (
                habitat,
                dispersal_sampler,
                turnover_rate,
                speciation_probability,
                origin_sampler_auxiliary,
                decomposition_auxiliary,
            ) = scenario.build::<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = ConditionalCoalescenceSampler::default();

            let decomposition = O::decompose(
                &habitat,
                local_partition.get_partition(),
                decomposition_auxiliary,
            );
            let origin_sampler = DecompositionOriginSampler::new(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                &decomposition,
            );

            let (lineage_store, dispersal_sampler, event_sampler, active_lineage_sampler): (
                O::LineageStore<GillespieLineageStore<M, O::Habitat>>,
                _,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                origin_sampler,
                dispersal_sampler,
                &coalescence_sampler,
                &turnover_rate,
                &speciation_probability,
                local_partition,
            )?;

            let emigration_exit = DomainEmigrationExit::new(decomposition);
            let immigration_entry = BufferedImmigrationEntry::default();

            let mut simulation = SimulationBuilder {
                maths: PhantomData::<M>,
                habitat,
                lineage_reference: PhantomData::<InMemoryLineageReference>,
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

            let (_status, time, steps) = match non_monolithic_parallelism_mode {
                ParallelismMode::Monolithic => unsafe { unreachable_unchecked() },
                ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => {
                    parallelisation::monolithic::optimistic::simulate(
                        &mut simulation,
                        delta_sync,
                        local_partition,
                    )
                },
                ParallelismMode::Lockstep => parallelisation::monolithic::lockstep::simulate(
                    &mut simulation,
                    local_partition,
                ),
                ParallelismMode::OptimisticLockstep => {
                    parallelisation::monolithic::optimistic_lockstep::simulate(
                        &mut simulation,
                        local_partition,
                    )
                },
                ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => {
                    parallelisation::monolithic::averaging::simulate(
                        &mut simulation,
                        delta_sync,
                        local_partition,
                    )
                },
            };

            // TODO: Adapt for parallel pausing
            Ok(AlgorithmResult::Done { time, steps })
        },
    }
}

#[allow(clippy::type_complexity)]
trait EventSkippingLineageStoreSampleInitialiser<
    M: MathsCore,
    G: RngCore<M>,
    O: Scenario<M, G>,
    Error,
> where
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    type DispersalSampler: SeparableDispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
    >: ActiveLineageSampler<
        M,
        O::Habitat,
        G,
        R,
        S,
        X,
        Self::DispersalSampler,
        ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
        O::TurnoverRate,
        O::SpeciationProbability,
        ConditionalGillespieEventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        R: LineageReference<M, O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
        >,
        coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
        turnover_rate: &O::TurnoverRate,
        speciation_probability: &O::SpeciationProbability,
        local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            ConditionalGillespieEventSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                O::TurnoverRate,
                O::SpeciationProbability,
            >,
            Self::ActiveLineageSampler<R, S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}
