use std::{fmt, mem::ManuallyDrop, rc::Rc};

use necsim_core::reporter::{boolean::True, Reporter};

use crate::{
    export::{ReporterPluginFilter, UnsafeReporterPlugin},
    import::serde::PluginLibrary,
};

#[allow(clippy::module_name_repetitions)]
pub struct ReporterPlugin {
    pub(crate) library: Rc<PluginLibrary>,

    pub(crate) reporter: ManuallyDrop<
        Box<dyn Reporter<ReportSpeciation = True, ReportDispersal = True, ReportProgress = True>>,
    >,
    pub(crate) filter: ReporterPluginFilter,
}

impl Drop for ReporterPlugin {
    fn drop(&mut self) {
        unsafe {
            (self.library.declaration.drop)(ManuallyDrop::new(UnsafeReporterPlugin {
                reporter: ManuallyDrop::take(&mut self.reporter),
                filter: self.filter,
            }))
        }
    }
}

impl fmt::Debug for ReporterPlugin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&*self.reporter, fmt)
    }
}
