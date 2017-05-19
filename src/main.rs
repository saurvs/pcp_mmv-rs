#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
pub mod pcp_mmv;

extern crate rand;

use pcp_mmv::*;
use rand::random;
use std::ffi::CString;
use std::mem;
use std::thread;
use std::time::Duration;

fn array_from_cstring(cstring: &CString) -> [i8; 64] {
    let mut array = [0i8; 64];
    for (i, chr) in cstring.as_bytes_with_nul().iter().enumerate() {
        array[i] = *chr as i8;
    }
    array
}

impl mmv_metric_t {
    pub unsafe fn new(
        name: &CString, item: u32, type_: mmv_metric_type_t,
        semantics: mmv_metric_sem_t, dimension: pmUnits, indom: u32,
        shorttext: &str, helptext: &str) -> mmv_metric_t {

        let shorttext = CString::new(shorttext).unwrap();
        let helptext = CString::new(helptext).unwrap();
        mmv_metric_t {
            name: array_from_cstring(name),
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

const PRODUCTS_COUNT: usize = 3;
const CLUSTER_ID: i32 = 321;
const PRODUCTS_INDOM: u32 = 61;
const MMV_FLAGS: mmv_stats_flags_t = mmv_stats_flags_t::MMV_FLAG_PROCESS;

pub fn main() {
    unsafe {
        let app_name = CString::new("acme").unwrap();

        /*
         * create indom
         */

        let product_external_1 = CString::new("Anvils").unwrap();
        let product_external_2 = CString::new("Rockets").unwrap();
        let product_external_3 = CString::new("Giant_Rubber_Bands").unwrap();

        let mut products: [mmv_instances_t; PRODUCTS_COUNT] = [
            mmv_instances_t { internal: 0, external: array_from_cstring(&product_external_1) },
            mmv_instances_t { internal: 1, external: array_from_cstring(&product_external_2) },
            mmv_instances_t { internal: 2, external: array_from_cstring(&product_external_3) },
        ];

        let indom_shorttext = CString::new("Acme products").unwrap();
        let indom_helptext = CString::new("Most popular products produced by the Acme Corporation").unwrap();

        let indom = mmv_indom_t {
            serial: PRODUCTS_INDOM,
            count: products.len() as u32,
            instances: products.as_mut_ptr(),
            shorttext: indom_shorttext.into_raw(),
            helptext: indom_helptext.into_raw()
        };
       
        /*
         * create count metric
         */
        
        let count_metric_name = CString::new("products.count").unwrap();

        let mut count_dimension: pmUnits = mem::zeroed();
        count_dimension.set_dimCount(1);
        count_dimension.set_scaleCount(PM_COUNT_ONE as i32);

        let count_metric = mmv_metric_t::new(
            &count_metric_name, // name
            7, // item
            mmv_metric_type_t::MMV_TYPE_U64,
            mmv_metric_sem_t::MMV_SEM_COUNTER,
            count_dimension,
            PRODUCTS_INDOM,
            "Acme factory product throughput",
            "Monotonic increasing counter of products produced in the Acme Corporation\nfactory since starting the Acme production application. Quality guaranteed."
        );

        /*
         * create time metric
         */

        let time_metric_name = CString::new("products.time").unwrap();

        let mut time_dimension: pmUnits = mem::zeroed();
        time_dimension.set_dimTime(1);
        time_dimension.set_scaleTime(PM_TIME_USEC);

        let time_metric = mmv_metric_t::new(
            &time_metric_name, // name
            8, // item
            mmv_metric_type_t::MMV_TYPE_U64,
            mmv_metric_sem_t::MMV_SEM_COUNTER,
            time_dimension,
            PRODUCTS_INDOM,
            "Machine time spent producing Acme products",
            "Machine time spent producing Acme Corporation products.  Does not include\ntime in queues waiting for production machinery."
        );

        /*
         * create queue metric
         */

        let queue_metric_name = CString::new("products.queuetime").unwrap();

        let queue_metric = mmv_metric_t::new(
            &queue_metric_name, // name
            10, // item
            mmv_metric_type_t::MMV_TYPE_U64,
            mmv_metric_sem_t::MMV_SEM_COUNTER,
            time_dimension, // reuse time_dimension
            PRODUCTS_INDOM,
            "Queued time while producing Acme products",
            "Time spent in the queue waiting to build Acme Corporation products,\nwhile some other Acme product was being built instead of this one."
        );

        /*
         * start mmv
         */

        let metrics = [count_metric, time_metric, queue_metric];
        let indoms = [indom];

        let mmv_handle = mmv_stats_init(
            app_name.as_ptr(), CLUSTER_ID, MMV_FLAGS,
            metrics.as_ptr(), metrics.len() as i32,
            indoms.as_ptr(), indoms.len() as i32,
        );

        /*
         * begin production!
         */

        let mut product;
        let mut working;
        let mut count: [*mut pmAtomValue; PRODUCTS_COUNT] = mem::zeroed();
        let mut machine: [*mut pmAtomValue; PRODUCTS_COUNT] = mem::zeroed();
        let mut inqueue: [*mut pmAtomValue; PRODUCTS_COUNT] = mem::zeroed();

        for (i, product) in products.iter().enumerate() {
            count[i] = mmv_lookup_value_desc(
                mmv_handle, count_metric_name.as_ptr(), product.external.as_ptr());
            machine[i] = mmv_lookup_value_desc(
                mmv_handle, time_metric_name.as_ptr(), product.external.as_ptr());
            inqueue[i] = mmv_lookup_value_desc(
                mmv_handle, queue_metric_name.as_ptr(), product.external.as_ptr());
        }

        loop {
            product = random::<usize>() % PRODUCTS_COUNT;
            working = random::<u64>() % 3;
            thread::sleep(Duration::from_secs(working));

            mmv_inc_value(mmv_handle, machine[product], 1.0);
            *(*count[product]).ull.as_mut() += 1;

            for i in 0..PRODUCTS_COUNT {
                if i != product {
                    mmv_inc_value(mmv_handle, inqueue[i], working as f64);
                }
            }
        }
    }
}