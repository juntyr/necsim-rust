use std::mem::ManuallyDrop;

use necsim_core::reporter::{
    boolean::{Boolean, True},
    Reporter,
};

pub struct ReporterPluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,

    pub init: unsafe extern "C" fn(&'static dyn log::Log, log::LevelFilter),

    pub deserialise:
        unsafe extern "C" fn(
            &mut dyn erased_serde::Deserializer,
        )
            -> Result<ManuallyDrop<UnsafeReporterPlugin>, erased_serde::Error>,

    pub drop: unsafe extern "C" fn(ManuallyDrop<UnsafeReporterPlugin>),
}

#[derive(Copy, Clone)]
pub struct ReporterPluginFilter {
    pub(crate) report_speciation: bool,
    pub(crate) report_dispersal: bool,
    pub(crate) report_progress: bool,
}

impl ReporterPluginFilter {
    #[must_use]
    pub fn from_reporter<R: Reporter>() -> Self {
        Self {
            report_speciation: R::ReportSpeciation::VALUE,
            report_dispersal: R::ReportDispersal::VALUE,
            report_progress: R::ReportProgress::VALUE,
        }
    }
}

#[repr(C)]
pub struct UnsafeReporterPlugin {
    pub(crate) reporter:
        Box<dyn Reporter<ReportSpeciation = True, ReportDispersal = True, ReportProgress = True>>,

    pub(crate) filter: ReporterPluginFilter,
}

impl<R: Reporter> From<R> for UnsafeReporterPlugin {
    fn from(reporter: R) -> Self {
        let boxed_reporter: Box<
            dyn Reporter<
                ReportSpeciation = R::ReportSpeciation,
                ReportDispersal = R::ReportDispersal,
                ReportProgress = R::ReportProgress,
            >,
        > = Box::new(reporter);

        Self {
            reporter: unsafe { std::mem::transmute(boxed_reporter) },
            filter: ReporterPluginFilter::from_reporter::<R>(),
        }
    }
}

#[macro_export]
macro_rules! export_plugin {
    ($($name:ident => $plugin:ty),+$(,)?) => {
        #[doc(hidden)]
        extern "C" fn __necsim_reporter_plugin_init(
            log: &'static dyn $crate::log::Log,
            max_level: $crate::log::LevelFilter,
        ) {
            let _ = $crate::log::set_logger(log);

            $crate::log::set_max_level(max_level);
        }

        #[doc(hidden)]
        extern "C" fn __necsim_reporter_plugin_deserialise<'de>(
            deserializer: &mut dyn $crate::erased_serde::Deserializer<'de>,
        ) -> Result<
            ::std::mem::ManuallyDrop<$crate::export::UnsafeReporterPlugin>,
            $crate::erased_serde::Error,
        > {
            #[allow(clippy::enum_variant_names)]
            #[derive($crate::serde::Deserialize)]
            #[serde(crate = "::necsim_plugins_core::serde")]
            enum Reporters {
                $($name($plugin)),*
            }

            $crate::erased_serde::deserialize::<Reporters>(deserializer).map(|reporter| {
                match reporter {
                    $(Reporters::$name(reporter) => reporter.into()),*
                }
            }).map(::std::mem::ManuallyDrop::new)
        }

        #[doc(hidden)]
        extern "C" fn __necsim_reporter_plugin_drop(
            plugin: ::std::mem::ManuallyDrop<$crate::export::UnsafeReporterPlugin>,
        ) {
            ::std::mem::drop(::std::mem::ManuallyDrop::into_inner(plugin))
        }

        #[doc(hidden)]
        #[no_mangle]
        pub static NECSIM_REPORTER_PLUGIN_DECLARATION: $crate::export::ReporterPluginDeclaration =
            $crate::export::ReporterPluginDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                core_version: $crate::CORE_VERSION,

                init: __necsim_reporter_plugin_init,
                deserialise: __necsim_reporter_plugin_deserialise,
                drop: __necsim_reporter_plugin_drop,
            };
    };
}
