#[macro_export]
macro_rules! explicit_landscape_total_habitat_contract {
    ($landscape:ident) => {{
        let extent = $landscape.get_extent();

        let mut total_habitat: usize = 0;

        for y in extent.y()..(extent.y() + extent.height()) {
            for x in extent.x()..(extent.x() + extent.width()) {
                total_habitat += $landscape.get_habitat_at_location(&Location::new(x, y)) as usize;
            }
        }

        total_habitat
    }};
}
