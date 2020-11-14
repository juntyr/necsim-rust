#![deny(clippy::pedantic)]
#![no_std]

pub type Habitat = necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat;
pub type Rng =
    necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>;
pub type DispersalSampler = necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler<Habitat, Rng>;
pub type LineageReference =
    necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference;
pub type LineageStore =
    necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore<
        Habitat,
    >;
pub type CoalescenceSampler =
    necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler<
        Habitat,
        Rng,
        LineageReference,
        LineageStore,
    >;
pub type EventSampler =
    necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler<
        Habitat,
        Rng,
        DispersalSampler,
        LineageReference,
        LineageStore,
    >;
pub type ActiveLineageSampler =
    necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler<
        Habitat,
        Rng,
        DispersalSampler,
        LineageReference,
        LineageStore,
    >;

pub type Simulation = necsim_core::simulation::Simulation<
    Habitat,
    Rng,
    DispersalSampler,
    LineageReference,
    LineageStore,
    CoalescenceSampler,
    EventSampler,
    ActiveLineageSampler,
>;

pub type EventBufferCudaRepresentation =
    necsim_impls_cuda::event_buffer::common::EventBufferCudaRepresentation<
        Habitat,
        LineageReference,
        true,  // REPORT_SPECIATION
        false, // REPORT_DISPERSAL
    >;
