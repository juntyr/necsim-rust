/// Inspired by the <https://adventures.michaelfbryan.com/posts/plugins-in-rust/> blog post
use std::{
    convert::TryFrom, fmt, io, iter::IntoIterator, mem::ManuallyDrop, path::PathBuf, rc::Rc,
};

use libloading::Library;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::{export::ReporterPluginDeclaration, import::ReporterPlugin};

pub struct ReporterPluginLibrary {
    _library: Rc<PluginLibrary>,
    reporters: Vec<ReporterPlugin>,
}

impl IntoIterator for ReporterPluginLibrary {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = ReporterPlugin;

    fn into_iter(self) -> Self::IntoIter {
        self.reporters.into_iter()
    }
}

impl<'de> Deserialize<'de> for ReporterPluginLibrary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["library", "reporters"];
        deserializer.deserialize_struct("Plugin", FIELDS, ReporterPluginLibraryVisitor)
    }
}

// Helper struct to load the library from its path
#[derive(serde::Deserialize)]
#[serde(try_from = "PathBuf")]
pub(crate) struct PluginLibrary {
    pub(crate) path: PathBuf,
    pub(crate) _library: Library,
    pub(crate) declaration: ReporterPluginDeclaration,
}

impl TryFrom<PathBuf> for PluginLibrary {
    type Error = io::Error;

    fn try_from(library_path: PathBuf) -> Result<Self, Self::Error> {
        // Load the plugin library into memory
        let library = unsafe { Library::new(library_path.clone()) }
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        // Load the plugin declaration symbol
        let declaration = unsafe {
            library
                .get::<*const ReporterPluginDeclaration>(b"NECSIM_REPORTER_PLUGIN_DECLARATION")
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
                .read()
        };

        // Check for rustc version incompatibilities
        if declaration.rustc_version != crate::RUSTC_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Plugin rustc version {} does not match system rustc version {}.",
                    declaration.rustc_version,
                    crate::RUSTC_VERSION
                ),
            ));
        }

        // Check for plugin system version incompatibilities
        if declaration.core_version != crate::CORE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Plugin system version {} does not match system version {}.",
                    declaration.core_version,
                    crate::CORE_VERSION
                ),
            ));
        }

        unsafe {
            (declaration.init)(log::logger(), log::max_level());
        }

        let path = unsafe { (declaration.library_path)() }.unwrap_or(library_path);

        Ok(Self {
            path,
            _library: library,
            declaration,
        })
    }
}

struct RcPluginLibrary(Rc<PluginLibrary>);

// Deserialise a list of ReporterPlugins using the open library
impl<'de> serde::de::DeserializeSeed<'de> for RcPluginLibrary {
    type Value = Vec<ReporterPlugin>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        // Helper struct to deserialise a single ReporterPlugin
        struct PluginReporter {
            library: Rc<PluginLibrary>,
        }

        impl<'de> serde::de::DeserializeSeed<'de> for PluginReporter {
            type Value = ReporterPlugin;

            fn deserialize<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                match unsafe {
                    (self.library.declaration.deserialise)(
                        &mut <dyn erased_serde::Deserializer>::erase(deserializer),
                    )
                } {
                    Ok(reporter) => Ok(ReporterPlugin {
                        library: self.library,
                        filter: reporter.filter,
                        reporter: ManuallyDrop::new(ManuallyDrop::into_inner(reporter).reporter),
                        finalised: false,
                    }),
                    Err(err) => Err(de::Error::custom(err)),
                }
            }
        }

        // Helper struct to deserialise a list of ReporterPlugins
        struct ReporterVecVisitor<'a> {
            library: Rc<PluginLibrary>,
            vec: &'a mut Vec<ReporterPlugin>,
        }

        impl<'de, 'a> Visitor<'de> for ReporterVecVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of reporters")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                while let Some(elem) = seq.next_element_seed(PluginReporter {
                    library: self.library.clone(),
                })? {
                    self.vec.push(elem);
                }

                Ok(())
            }
        }

        let mut reporters = Vec::new();

        deserializer.deserialize_seq(ReporterVecVisitor {
            library: self.0,
            vec: &mut reporters,
        })?;

        Ok(reporters)
    }
}

// Helper enum to deserialise field names
#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum ReporterPluginLibraryField {
    Library,
    Reporters,
}

// Helper struct to sequentially load the library, then the plugins
struct ReporterPluginLibraryVisitor;

impl<'de> Visitor<'de> for ReporterPluginLibraryVisitor {
    type Value = ReporterPluginLibrary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Plugin")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<ReporterPluginLibrary, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let library: Rc<PluginLibrary> = Rc::new(
            seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?,
        );

        let reporters: Vec<ReporterPlugin> = seq
            .next_element_seed(RcPluginLibrary(library.clone()))?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        Ok(ReporterPluginLibrary {
            _library: library,
            reporters,
        })
    }

    fn visit_map<V>(self, mut map: V) -> Result<ReporterPluginLibrary, V::Error>
    where
        V: MapAccess<'de>,
    {
        let library: Rc<PluginLibrary> =
            if let Some(ReporterPluginLibraryField::Library) = map.next_key()? {
                Rc::new(map.next_value()?)
            } else {
                return Err(de::Error::missing_field("library"));
            };

        let reporters: Vec<ReporterPlugin> =
            if let Some(ReporterPluginLibraryField::Reporters) = map.next_key()? {
                map.next_value_seed(RcPluginLibrary(library.clone()))?
            } else {
                return Err(de::Error::missing_field("reporters"));
            };

        Ok(ReporterPluginLibrary {
            _library: library,
            reporters,
        })
    }
}
