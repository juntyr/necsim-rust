use std::{fmt, mem::ManuallyDrop, rc::Rc};

use necsim_core::reporter::{boolean::True, Reporter};

use crate::{export::UnsafeReporterPlugin, import::PluginLibrary};

pub struct ReporterPlugin {
    pub(crate) library: Rc<PluginLibrary>,

    pub(crate) reporter: ManuallyDrop<
        Box<dyn Reporter<ReportSpeciation = True, ReportDispersal = True, ReportProgress = True>>,
    >,

    pub(crate) report_speciation: bool,
    pub(crate) report_dispersal: bool,
    pub(crate) report_progress: bool,
}

impl Drop for ReporterPlugin {
    fn drop(&mut self) {
        unsafe {
            (self.library.declaration.drop)(ManuallyDrop::new(UnsafeReporterPlugin {
                reporter: ManuallyDrop::take(&mut self.reporter),

                report_speciation: self.report_speciation,
                report_dispersal: self.report_dispersal,
                report_progress: self.report_progress,
            }))
        }
    }
}

impl fmt::Debug for ReporterPlugin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&*self.reporter, fmt)
    }
}
