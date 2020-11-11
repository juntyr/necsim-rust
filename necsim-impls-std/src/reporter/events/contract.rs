use necsim_core::cogs::{Habitat, LineageReference};
use necsim_core::event::EventType;
use necsim_core::landscape::IndexedLocation;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::too_many_arguments)]
pub fn explicit_event_reporter_report_event_contract<H: Habitat, R: LineageReference<H>>(
    dispersal_origin: &IndexedLocation,
    event_type: &EventType<H, R>,
    old_speciation: usize,
    old_out_dispersal: usize,
    old_self_dispersal: usize,
    old_out_coalescence: usize,
    old_self_coalescence: usize,
    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
) -> bool {
    match event_type {
        EventType::Speciation => {
            speciation == old_speciation + 1 &&
            out_dispersal == old_out_dispersal &&
            self_dispersal == old_self_dispersal &&
            out_coalescence == old_out_coalescence &&
            self_coalescence == old_self_coalescence
        },
        EventType::Dispersal {
            target,
            coalescence: None,
            ..
        } if dispersal_origin == target => {
            speciation == old_speciation &&
            out_dispersal == old_out_dispersal &&
            self_dispersal == old_self_dispersal + 1 &&
            out_coalescence == old_out_coalescence &&
            self_coalescence == old_self_coalescence
        },
        EventType::Dispersal {
            coalescence: None,
            ..
        } /*if origin != target*/ => {
            speciation == old_speciation &&
            out_dispersal == old_out_dispersal + 1 &&
            self_dispersal == old_self_dispersal &&
            out_coalescence == old_out_coalescence &&
            self_coalescence == old_self_coalescence
        },
        EventType::Dispersal {
            target,
            coalescence: Some(_),
            ..
        } if dispersal_origin == target => {
            speciation == old_speciation &&
            out_dispersal == old_out_dispersal &&
            self_dispersal == old_self_dispersal &&
            out_coalescence == old_out_coalescence &&
            self_coalescence == old_self_coalescence + 1
        },
        EventType::Dispersal {
            coalescence: Some(_),
            ..
        } /*if origin != target*/ => {
            speciation == old_speciation &&
            out_dispersal == old_out_dispersal &&
            self_dispersal == old_self_dispersal &&
            out_coalescence == old_out_coalescence + 1 &&
            self_coalescence == old_self_coalescence
        },
    }
}
