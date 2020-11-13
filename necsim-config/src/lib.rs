#[macro_export]
macro_rules! RngType {
    () => {
        necsim_impls_no_std::cogs::rng::wyhash::WyHash
    };
}

#[macro_export]
macro_rules! ReporterType {
    (< $Habitat:ty, $LineageReference:ty >) => {
        necsim_core::ReporterGroupType! {
            <$Habitat, $LineageReference>[
                necsim_impls_std::reporter::biodiversity::BiodiversityReporter,
                necsim_impls_std::reporter::progress::ProgressReporter
            ]
        }
    };
}
