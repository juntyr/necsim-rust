use std::{hint::unreachable_unchecked, marker::PhantomData};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, ImmigrationEntry, LineageReference,
        LocallyCoherentLineageStore, MathsCore, RngCore, SplittableRng,
    },
    event::DispersalEvent,
    lineage::{Lineage, LineageInteraction},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::{
            classical::ClassicalActiveLineageSampler,
            resuming::{
                ExceptionalLineage, RestartFixUpActiveLineageSampler, SplitExceptionalLineages,
            },
        },
        coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
        dispersal_sampler::{
            in_memory::alias::InMemoryAliasDispersalSampler,
            trespassing::{
                uniform::UniformAntiTrespassingDispersalSampler, TrespassingDispersalSampler,
            },
        },
        emigration_exit::{domain::DomainEmigrationExit, never::NeverEmigrationExit},
        event_sampler::unconditional::UnconditionalEventSampler,
        immigration_entry::{buffered::BufferedImmigrationEntry, never::NeverImmigrationEntry},
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::locally::classical::ClassicalLineageStore,
        maths::intrinsics::IntrinsicsMathsCore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
            resuming::ResumingOriginSampler, TrustedOriginSampler,
        },
        turnover_rate::uniform::UniformTurnoverRate,
    },
    parallelisation::{self, Status},
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{
    Algorithm, AlgorithmResult, CoalescenceStrategy, ContinueError, OutOfDemeStrategy,
    OutOfHabitatStrategy, RestartFixUpStrategy,
};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{
    AveragingParallelismMode, MonolithicArguments, OptimisticParallelismMode, ParallelismMode,
};

use super::GillespieAlgorithm;

// Optimised 'Classical' implementation for the `UniformTurnoverSampler`
#[allow(clippy::type_complexity)]
impl<
        O: Scenario<
            IntrinsicsMathsCore,
            Pcg<IntrinsicsMathsCore>,
            LineageReference = InMemoryLineageReference,
            TurnoverRate = UniformTurnoverRate,
        >,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for GillespieAlgorithm
where
    O::LineageStore<ClassicalLineageStore<IntrinsicsMathsCore, O::Habitat>>:
        LocallyCoherentLineageStore<IntrinsicsMathsCore, O::Habitat, InMemoryLineageReference>,
{
    #[allow(clippy::too_many_lines)]
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
            ClassicalLineageStoreSampleInitialiser<M, G, O, !> for GenesisInitialiser
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
            > = ClassicalActiveLineageSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                O::SpeciationProbability,
                I,
            >;
            type DispersalSampler =
                O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemoryAliasDispersalSampler<M, O::Habitat, G>,
                >,
                _local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                !,
            >
            where
                O::Habitat: 'h,
            {
                let (lineage_store, active_lineage_sampler) =
                    ClassicalActiveLineageSampler::init_with_store(origin_sampler);

                Ok((lineage_store, dispersal_sampler, active_lineage_sampler))
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
            > ClassicalLineageStoreSampleInitialiser<M, G, O, ContinueError<!>>
            for ResumeInitialiser<L>
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
            > = ClassicalActiveLineageSampler<
                M,
                O::Habitat,
                G,
                R,
                S,
                X,
                Self::DispersalSampler,
                O::SpeciationProbability,
                I,
            >;
            type DispersalSampler =
                O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemoryAliasDispersalSampler<M, O::Habitat, G>,
                >,
                _local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                ContinueError<!>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let (lineage_store, active_lineage_sampler, exceptional_lineages) =
                    ClassicalActiveLineageSampler::resume_with_store(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                        self.resume_after.unwrap_or(NonNegativeF64::zero()),
                    );

                if !exceptional_lineages.is_empty() {
                    return Err(ContinueError::Sample(exceptional_lineages));
                }

                Ok((lineage_store, dispersal_sampler, active_lineage_sampler))
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
            > ClassicalLineageStoreSampleInitialiser<M, G, O, ContinueError<!>>
            for FixUpInitialiser<L>
        {
            type ActiveLineageSampler<
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
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
                UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
                UniformTurnoverRate,
                O::SpeciationProbability,
                UnconditionalEventSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
                    UniformTurnoverRate,
                    O::SpeciationProbability,
                >,
                I,
                ClassicalActiveLineageSampler<
                    M,
                    O::Habitat,
                    G,
                    R,
                    S,
                    X,
                    Self::DispersalSampler,
                    O::SpeciationProbability,
                    I,
                >,
            >;
            type DispersalSampler = TrespassingDispersalSampler<
                M,
                O::Habitat,
                G,
                O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, G>,
            >;

            fn init<
                'h,
                T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
                R: LineageReference<M, O::Habitat>,
                S: LocallyCoherentLineageStore<M, O::Habitat, R>,
                X: EmigrationExit<M, O::Habitat, G, R, S>,
                I: ImmigrationEntry<M>,
                Q: Reporter,
                P: LocalPartition<Q>,
            >(
                self,
                origin_sampler: T,
                dispersal_sampler: O::DispersalSampler<
                    InMemoryAliasDispersalSampler<M, O::Habitat, G>,
                >,
                local_partition: &mut P,
            ) -> Result<
                (
                    S,
                    Self::DispersalSampler,
                    Self::ActiveLineageSampler<R, S, X, I>,
                ),
                ContinueError<!>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let (lineage_store, active_lineage_sampler, exceptional_lineages) =
                    ClassicalActiveLineageSampler::resume_with_store(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
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

                let dispersal_sampler = TrespassingDispersalSampler::new(
                    dispersal_sampler,
                    UniformAntiTrespassingDispersalSampler::default(),
                );
                let active_lineage_sampler = RestartFixUpActiveLineageSampler::new(
                    active_lineage_sampler,
                    fixable_lineages,
                    self.restart_at,
                );

                Ok((lineage_store, dispersal_sampler, active_lineage_sampler))
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

#[allow(clippy::too_many_lines)]
fn initialise_and_simulate<
    M: MathsCore,
    G: SplittableRng<M>,
    O: Scenario<M, G, LineageReference = InMemoryLineageReference, TurnoverRate = UniformTurnoverRate>,
    R: Reporter,
    P: LocalPartition<R>,
    I: Iterator<Item = u64>,
    L: ClassicalLineageStoreSampleInitialiser<M, G, O, Error>,
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
    O::LineageStore<ClassicalLineageStore<M, O::Habitat>>:
        LocallyCoherentLineageStore<M, O::Habitat, InMemoryLineageReference>,
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
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = UnconditionalCoalescenceSampler::default();
            let event_sampler = UnconditionalEventSampler::default();

            let (lineage_store, dispersal_sampler, active_lineage_sampler): (
                O::LineageStore<ClassicalLineageStore<M, O::Habitat>>,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                dispersal_sampler,
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
            ) = scenario.build::<InMemoryAliasDispersalSampler<M, O::Habitat, G>>();
            let coalescence_sampler = UnconditionalCoalescenceSampler::default();
            let event_sampler = UnconditionalEventSampler::default();

            let decomposition = O::decompose(
                &habitat,
                local_partition.get_partition(),
                decomposition_auxiliary,
            );
            let origin_sampler = DecompositionOriginSampler::new(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                &decomposition,
            );

            let (lineage_store, dispersal_sampler, active_lineage_sampler): (
                O::LineageStore<ClassicalLineageStore<M, O::Habitat>>,
                _,
                _,
            ) = lineage_store_sampler_initialiser.init(
                origin_sampler,
                dispersal_sampler,
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
trait ClassicalLineageStoreSampleInitialiser<M: MathsCore, G: RngCore<M>, O: Scenario<M, G>, Error>
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
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
        UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
        UniformTurnoverRate,
        O::SpeciationProbability,
        UnconditionalEventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
            UniformTurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
        local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<R, S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}
