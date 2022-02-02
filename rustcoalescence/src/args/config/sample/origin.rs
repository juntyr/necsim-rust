use std::{collections::HashSet, fmt};

use fnv::FnvBuildHasher;
use serde::{Deserialize, Deserializer};
use serde_state::DeserializeState;

use necsim_core::lineage::Lineage;
use necsim_impls_std::lineage_file::loader::LineageFileLoader;

use super::{
    super::pause::{Pause, SampleDestiny},
    SampleOrigin,
};

impl fmt::Display for SampleOrigin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Habitat => fmt.write_str("Habitat"),
            Self::List(_) => fmt.write_str("List"),
            Self::Bincode(_) => fmt.write_str("Bincode"),
        }
    }
}

impl fmt::Debug for SampleOrigin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct VecLineages(usize);

        impl fmt::Debug for VecLineages {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "Vec<Lineage; {}>", self.0)
            }
        }

        match self {
            Self::Habitat => fmt.debug_struct(stringify!(Habitat)).finish(),
            Self::List(lineages) => fmt
                .debug_tuple(stringify!(List))
                .field(&VecLineages(lineages.len()))
                .finish(),
            Self::Bincode(loader) => fmt
                .debug_tuple(stringify!(Bincode))
                .field(&VecLineages(loader.get_lineages().len()))
                .finish(),
        }
    }
}

impl<'de> DeserializeState<'de, &'de Option<Pause>> for SampleOrigin {
    fn deserialize_state<D: Deserializer<'de>>(
        pause: &mut &'de Option<Pause>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw = SampleOriginRaw::deserialize(deserializer)?;

        if let Some(pause) = pause {
            if matches!(pause.destiny, SampleDestiny::List)
                && !matches!(raw, SampleOriginRaw::List(_))
            {
                return Err(serde::de::Error::custom(format!(
                    "`List` pause destiny requires `List` origin sample, found `{}`",
                    raw
                )));
            }
        }

        let lineages = match &raw {
            SampleOriginRaw::Habitat => return Ok(Self::Habitat),
            SampleOriginRaw::List(lineages) => lineages.iter(),
            SampleOriginRaw::Bincode(loader) => loader.get_lineages().iter(),
        };

        let mut global_references =
            HashSet::with_capacity_and_hasher(lineages.len(), FnvBuildHasher::default());

        for lineage in lineages {
            if !global_references.insert(lineage.global_reference.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate lineage reference #{}",
                    lineage.global_reference
                )));
            }
        }

        match raw {
            SampleOriginRaw::Habitat => Ok(Self::Habitat),
            SampleOriginRaw::List(lineages) => Ok(Self::List(lineages)),
            SampleOriginRaw::Bincode(loader) => Ok(Self::Bincode(loader)),
        }
    }
}

#[derive(Debug, Deserialize)]
enum SampleOriginRaw {
    Habitat,
    List(Vec<Lineage>),
    Bincode(LineageFileLoader),
}

impl fmt::Display for SampleOriginRaw {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Habitat => fmt.write_str("Habitat"),
            Self::List(_) => fmt.write_str("List"),
            Self::Bincode(_) => fmt.write_str("Bincode"),
        }
    }
}
