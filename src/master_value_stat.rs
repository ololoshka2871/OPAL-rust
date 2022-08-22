use crate::time_base::master_counter::MasterTimerInfo;

struct MasterGetter {
    master: MasterTimerInfo,
}

impl MasterGetter {
    fn new(master: MasterTimerInfo) -> Self {
        Self { master }
    }

    fn value(&mut self) -> u32 {
        (self.master.value64().0 >> 4) as u32
    }
}

static mut MASTER_TIMER_VALUE_GETTER: Option<MasterGetter> = None;

#[allow(non_camel_case_types)]
#[no_mangle]
pub unsafe extern "C" fn getMaterCounterValue() -> u32 {
    if let Some(m) = MASTER_TIMER_VALUE_GETTER.as_mut() {
        m.value()
    } else {
        0
    }
}

pub fn init_master_getter(mut counter: MasterTimerInfo) {
    counter.want_start();
    unsafe {
        MASTER_TIMER_VALUE_GETTER = Some(MasterGetter::new(counter));
    };
}
