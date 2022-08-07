use std::mem::ManuallyDrop;

use necsim_core::reporter::{
    boolean::{Boolean, True},
    Reporter,
};

pub trait SerializeableReporter: Reporter + erased_serde::Serialize {
    fn reporter_name(&self) -> &'static str;
}

pub struct ReporterPluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,

    pub init: unsafe extern "C" fn(&'static dyn log::Log, &log::LevelFilter),

    // SOLUTION I : deserialiser wrapper which never calls
    //              - visit_string (visit_str instead)
    //              - visit_byte_buf (visit_bytes instead)

    // TODO: the value can also contain strings and vecs constructed on the main side by the
    // deserializer TODO: the error can be constructed both on the main side (deserialiser
    // error) or on the plugin side (derialisee error) and we have no clue which it is here
    pub deserialise:
        unsafe extern "C" fn(
            &mut dyn erased_serde::Deserializer,
        )
            -> Result<ManuallyDrop<UnsafeReporterPlugin>, erased_serde::Error>,

    pub library_path: unsafe extern "C" fn() -> Option<&'static ::std::path::Path>,

    pub drop: unsafe extern "C" fn(ManuallyDrop<UnsafeReporterPlugin>),
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ReporterPluginFilter {
    pub(crate) report_speciation: bool,
    pub(crate) report_dispersal: bool,
    pub(crate) report_progress: bool,
}

impl ReporterPluginFilter {
    #[must_use]
    pub fn from_reporter<R: SerializeableReporter>() -> Self {
        Self {
            report_speciation: R::ReportSpeciation::VALUE,
            report_dispersal: R::ReportDispersal::VALUE,
            report_progress: R::ReportProgress::VALUE,
        }
    }
}

pub type DynReporterPlugin = dyn SerializeableReporter<
    ReportSpeciation = True,
    ReportDispersal = True,
    ReportProgress = True,
>;

#[repr(C)]
pub struct UnsafeReporterPlugin {
    pub(crate) reporter: Box<DynReporterPlugin>,
    pub(crate) filter: ReporterPluginFilter,
}

impl<R: SerializeableReporter> From<R> for UnsafeReporterPlugin {
    fn from(reporter: R) -> Self {
        let boxed_reporter: Box<
            dyn SerializeableReporter<
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
#[allow(clippy::module_name_repetitions)]
macro_rules! export_plugin {
    ($($name:ident => $plugin:ty),+$(,)?) => {
        $(impl $crate::export::SerializeableReporter for $plugin {
            fn reporter_name(&self) -> &'static str {
                stringify!($name)
            }
        })*

        #[doc(hidden)]
        #[no_mangle]
        pub static NECSIM_REPORTER_PLUGIN_DECLARATION: $crate::export::ReporterPluginDeclaration = {
            extern "C" fn necsim_reporter_plugin_init(
                log: &'static dyn $crate::log::Log,
                max_level: &$crate::log::LevelFilter,
            ) {
                let _ = $crate::log::set_logger(log);

                $crate::log::set_max_level(*max_level);
            }

            extern "C" fn necsim_reporter_plugin_deserialise<'de>(
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

            extern "C" fn necsim_reporter_plugin_library_path() -> Option<&'static ::std::path::Path> {
                static LIBRARY_PATH_INIT: ::std::sync::Once = ::std::sync::Once::new();
                static mut LIBRARY_PATH_VAL: Option::<::std::path::PathBuf> = None;

                // Safety: LIBRARY_PATH_VAL is only mutated here, only once
                LIBRARY_PATH_INIT.call_once(|| unsafe {
                    LIBRARY_PATH_VAL = None;//::necsim_plugins_core::process_path::get_dylib_path();
                });

                // Safety: LIBRARY_PATH_VAL is only mutated once, always before
                unsafe { LIBRARY_PATH_VAL.as_deref() }
            }

            extern "C" fn necsim_reporter_plugin_drop(
                plugin: ::std::mem::ManuallyDrop<$crate::export::UnsafeReporterPlugin>,
            ) {
                ::std::mem::drop(::std::mem::ManuallyDrop::into_inner(plugin))
            }

            $crate::export::ReporterPluginDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                core_version: $crate::CORE_VERSION,

                init: necsim_reporter_plugin_init,
                deserialise: necsim_reporter_plugin_deserialise,
                library_path: necsim_reporter_plugin_library_path,
                drop: necsim_reporter_plugin_drop,
            }
        };
    };
}

pub enum Reporters<'r> {
    DynReporter(&'r DynReporterPlugin),
}

impl<'r> serde::Serialize for Reporters<'r> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Reporter<'r>(&'r DynReporterPlugin);

        impl<'r> serde::Serialize for Reporter<'r> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                erased_serde::serialize(self.0, serializer)
            }
        }

        let Self::DynReporter(reporter) = self;

        serializer.serialize_newtype_variant(
            "Reporters",
            0,
            reporter.reporter_name(),
            &Reporter(*reporter),
        )
    }
}
