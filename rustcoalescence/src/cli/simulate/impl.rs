#[macro_export]
macro_rules! match_scenario_algorithm {
    (
        ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $($(#[$meta:meta])* $algpat:pat => $algcode:block),*
            <=>
            $($scenpat:pat => $scencode:block),*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$meta])* $algpat => $algcode),*
                <=>
                $($scenpat => $scencode),*
                <=>
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $(#[$meta:meta])* $algpat:pat => $algcode:block,
            $($(#[$metarem:meta])* $algpatrem:pat => $algcoderem:block),+
            <=>
            $($scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$metarem])* $algpatrem => $algcoderem),+
                <=>
                $($scenpat => $scencode),*
                <=>
                $($tail)*
                $(#[$meta])* $algpat => {
                    match $scenario {
                        $($scenpat => {
                            let $algscen = $scencode;
                            $algcode
                        }),*
                    }
                }
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $(#[$meta:meta])* $algpat:pat => $algcode:block
            <=>
            $($scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match $algorithm {
            $($tail)*
            $(#[$meta])* $algpat => {
                match $scenario {
                    $($scenpat => {
                        let $algscen = $scencode;
                        $algcode
                    }),*
                }
            }
        }
    };
}
