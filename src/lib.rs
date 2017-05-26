#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod sys;

pub use self::sys::*;

use std::ffi::CString;
use std::mem;
use std::ptr;

fn array64bytes_from_cstring(cstring: &CString) -> [i8; 64] {
    let mut array = [0i8; 64];
    for (byte, chr) in array.iter_mut().zip(cstring.as_bytes_with_nul().iter()) {
        *byte = *chr as i8; 
    }
    array 
}

pub type Instance = mmv_instances_t;

impl Instance {
    pub fn new(internal: i32, external: &str) -> Instance {
        let ext = CString::new(external).unwrap();
        Instance {
            internal: internal,
            external: array64bytes_from_cstring(&ext),
        }
    }
}

pub type Indom = mmv_indom_t;

impl Indom {
    pub fn new(
        serial: u32, instances: &mut [Instance],
        shorttext: &str, helptext: &str) -> Indom {

        let shorttext = CString::new(shorttext).unwrap();
        let helptext = CString::new(helptext).unwrap();
        Indom {
            serial: serial,
            count: instances.len() as u32,
            instances: instances.as_mut_ptr(),
            shorttext: shorttext.into_raw(),
            helptext: helptext.into_raw()
        }
    }
}

pub type Metric = mmv_metric_t;
pub use self::sys::mmv_metric_type_t as MetricType;
pub use self::sys::mmv_metric_sem_t as MetricSem;

impl Metric {
    pub fn new(
        name: &str, item: u32, type_: MetricType,
        semantics: MetricSem, dimension: PMUnits, indom: u32,
        shorttext: &str, helptext: &str) -> Metric {

        let name = CString::new(name).unwrap();
        let shorttext = CString::new(shorttext).unwrap();
        let helptext = CString::new(helptext).unwrap();
        Metric {
            name: array64bytes_from_cstring(&name),
            item: item,
            type_: type_,
            semantics: semantics,
            dimension: dimension,
            indom: indom,
            shorttext: shorttext.into_raw(),
            helptext: helptext.into_raw()
        }
    }
}

pub type PMUnits = pmUnits;

impl PMUnits {
    pub fn new() -> PMUnits {
        unsafe {
            mem::zeroed()
        }
    }
}

#[derive(Clone, Copy)]
pub struct AtomValue {
    pm_value: *mut pmAtomValue
}

impl AtomValue {
    pub fn new() -> AtomValue {
        AtomValue {
            pm_value: ptr::null_mut()
        }
    }

    pub fn ull(&self) -> u64 {
        unsafe {
            *(*self.pm_value).ull.as_ref()
        }
    }

    pub fn set_ull(&mut self, val: u64) {
        unsafe {
            *(*self.pm_value).ull.as_mut() = val;
        }
    }

    pub fn inc_ull(&mut self, val: u64) {
        unsafe {
            *(*self.pm_value).ull.as_mut() += val;
        }
    }
}

pub struct MMV {
    handle: *mut ::std::os::raw::c_void
}

unsafe impl Send for MMV {}
unsafe impl Sync for MMV {}

impl MMV {
    pub fn new() -> MMV {
        MMV {
            handle: ptr::null_mut()
        }
    }

    pub fn start(
        &mut self, app_name: &str, cluster_id: i32, flags: u32,
        metrics: &[Metric], indoms: &[Indom]) {

        let app_name = CString::new(app_name).unwrap();
        self.handle = unsafe {
            sys::mmv_stats_init(
                app_name.as_ptr(), cluster_id, flags,
                metrics.as_ptr(), metrics.len() as i32,
                indoms.as_ptr(), indoms.len() as i32,
            )
        };
    }

    pub fn lookup_value(
        &self, metric_name: &str, instance: &str) -> AtomValue {
        
        let metric_name = CString::new(metric_name).unwrap();
        let instance = CString::new(instance).unwrap();
        let value = unsafe {
            mmv_lookup_value_desc(
                self.handle, metric_name.as_ptr(), instance.as_ptr())
        };
        AtomValue { pm_value: value }
    }

    pub fn inc_value(&self, value: &mut AtomValue, amt: f64) {
        unsafe {
            mmv_inc_value(self.handle, value.pm_value, amt)
        }
    }
}
