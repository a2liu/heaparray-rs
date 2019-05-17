use interloc::*;

pub struct TestMonitor {
    local: ThreadMonitor,
}

impl TestMonitor {
    // This needs to be const to be usable in static functions
    pub const fn new() -> Self {
        Self {
            local: ThreadMonitor::new(),
        }
    }

    pub fn local_info(&self) -> AllocInfo {
        self.local.info()
    }

    pub fn local_reset(&self) {
        self.local.write_info(AllocInfo::new());
    }
}

impl AllocMonitor for TestMonitor {
    fn monitor(&self, layout: Layout, action: AllocAction) {
        self.local.monitor(layout, action);
    }
}
