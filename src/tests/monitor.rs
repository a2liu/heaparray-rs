use interloc::*;

pub struct TestMonitor {
    global: StatsMonitor,
    local: ThreadMonitor,
}

impl TestMonitor {
    // This needs to be const to be usable in static functions
    pub const fn new() -> Self {
        Self {
            global: StatsMonitor::new(),
            local: ThreadMonitor::new(),
        }
    }

    pub fn global_info(&self) -> AllocInfo {
        self.global.info()
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
        self.global.monitor(layout, action);
        self.local.monitor(layout, action);
    }
}
