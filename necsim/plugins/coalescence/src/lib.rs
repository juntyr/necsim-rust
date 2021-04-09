#![deny(clippy::pedantic)]

// IDEA: produce the coalescence tree in the form of a queryable DB

// LINEAGES (
//    individual_id, /* should combine original indexed location + community */
//    species_id, /* should combine speciation indexed location + community +
//                   time */
//    community_original,
//    x_original,
//    y_original,
//    index_original,
//    parent, /* forwards in time, =self.individual_id if speciation source */
//    birth_time,
//    speciation_draw, /* U(0, 1) - speciation if < s */
// )
