#[macro_export]
macro_rules! impl_report {
    // Special case: Unused = MaybeUsed<False>
    ($(#[$metas:meta])* $target:ident(&mut $this:ident, $value:ident: Unused)
        -> Unused $code:block) =>
    {
        impl_report!{
            $(#[$metas])*
            $target(&mut $this, $value: Unused) -> MaybeUsed<$crate::reporter::boolean::False>
            $code
        }
    };
    // Special case: Used = MaybeUsed<True>
    ($(#[$metas:meta])* $target:ident(&mut $this:ident, $value:ident: Unused)
        -> Used $code:block) =>
    {
        impl_report!{
            $(#[$metas])*
            $target(&mut $this, $value: Unused) -> MaybeUsed<$crate::reporter::boolean::True>
            $code
        }
    };
    // Dispatch case: MaybeUsed + speciation
    ($(#[$metas:meta])* speciation(&mut $this:ident, $value:ident: Unused)
        -> MaybeUsed<$Usage:ty> $code:block) =>
    {
        impl_report!{
            $(#[$metas])*
            fn report_speciation(&mut $this, $value: Unused<$crate::event::SpeciationEvent>)
                -> MaybeUsed<$crate::event::SpeciationEvent, ReportSpeciation = $Usage>
            $code
        }
    };
    // Dispatch case: MaybeUsed + dispersal
    ($(#[$metas:meta])* dispersal(&mut $this:ident, $value:ident: Unused)
        -> MaybeUsed<$Usage:ty> $code:block) =>
    {
        impl_report!{
            $(#[$metas])*
            fn report_dispersal(&mut $this, $value: Unused<$crate::event::DispersalEvent>)
                -> MaybeUsed<$crate::event::DispersalEvent, ReportDispersal = $Usage>
            $code
        }
    };
    // Dispatch case: MaybeUsed + progress
    ($(#[$metas:meta])* progress(&mut $this:ident, $value:ident: Unused)
        -> MaybeUsed<$Usage:ty> $code:block) =>
    {
        impl_report!{
            $(#[$metas])*
            fn report_progress(&mut $this, $value: Unused<u64>)
                -> MaybeUsed<u64, ReportProgress = $Usage>
            $code
        }
    };
    // Impl case: MaybeUsed
    ($(#[$metas:meta])* fn $target:ident(&mut $this:ident, $value:ident: Unused<$tyi:ty>)
        -> MaybeUsed<$tyo:ty, $UsageIdent:ident = $UsageTy:ty> $code:block) =>
    {
        $(#[$metas])*
        fn $target<'a>(
            &mut $this,
            $value: $crate::reporter::used::Unused<'a, $tyi>
        ) -> $crate::reporter::used::MaybeUsed<'a, $tyo, Self::$UsageIdent> {
            $code
        }

        type $UsageIdent = $UsageTy;
    };
}

#[macro_export]
macro_rules! impl_finalise {
    ($(#[$metas:meta])* ($self:ident) $code:block) => {
        $(#[$metas])*
        fn finalise($self) where Self:Sized {
            $code
        }

        $(#[$metas])*
        unsafe fn finalise_boxed($self: $crate::alloc::boxed::Box<Self>) {
            $code
        }
    };
    ($(#[$metas:meta])* (mut $self:ident) $code:block) => {
        $(#[$metas])*
        fn finalise(mut $self) where Self:Sized {
            $code
        }

        $(#[$metas])*
        unsafe fn finalise_boxed(mut $self: $crate::alloc::boxed::Box<Self>) {
            $code
        }
    };
}
