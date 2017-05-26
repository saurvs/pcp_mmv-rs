extern crate pcp_mmv;
extern crate rand;

use pcp_mmv::*;
use rand::random;
use std::thread;
use std::time::Duration;

const PRODUCTS_COUNT: usize = 3;
const CLUSTER_ID: i32 = 321;
const PRODUCTS_INDOM: u32 = 61;
const MMV_FLAGS: u32 = 0;

pub fn main() {

    /*
     * create indom
     */

    let products = [
        "Anvils",
        "Rockets",
        "Giant_Rubber_Bands"
    ];

    let mut instances = [
        Instance::new(0, products[0]),
        Instance::new(1, products[1]),
        Instance::new(2, products[2]),
    ];

    let indom = Indom::new(
        PRODUCTS_INDOM, // serial
        &mut instances,
        "Acme products",
        "Most popular products produced by the Acme Corporation"
    );
    
    /*
     * create count metric
     */
    
    let count_metric_name = "products.count";

    let mut count_dimension = PMUnits::new();
    count_dimension.set_dimCount(1);
    count_dimension.set_scaleCount(PM_COUNT_ONE as i32);

    let count_metric = Metric::new(
        count_metric_name, // name
        7, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
        count_dimension,
        PRODUCTS_INDOM,
        "Acme factory product throughput",
        "Monotonic increasing counter of products produced in the Acme Corporation\nfactory since starting the Acme production application. Quality guaranteed."
    );

    /*
     * create time metric
     */

    let time_metric_name = "products.time";

    let mut time_dimension = PMUnits::new();
    time_dimension.set_dimTime(1);
    time_dimension.set_scaleTime(PM_TIME_USEC);

    let time_metric = Metric::new(
        time_metric_name, // name
        8, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
        time_dimension,
        PRODUCTS_INDOM,
        "Machine time spent producing Acme products",
        "Machine time spent producing Acme Corporation products.  Does not include\ntime in queues waiting for production machinery."
    );

    /*
     * create queue metric
     */

    let queue_metric_name = "products.queuetime";

    let queue_metric = Metric::new(
        queue_metric_name, // name
        10, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
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
    let mut mmv = MMV::new();

    mmv.start("acme", CLUSTER_ID, MMV_FLAGS, &metrics, &indoms);

    /*
     * begin production!
     */

    let mut count = [AtomValue::new(); PRODUCTS_COUNT];
    let mut machine = [AtomValue::new(); PRODUCTS_COUNT];
    let mut inqueue = [AtomValue::new(); PRODUCTS_COUNT];

    for (i, product) in products.iter().enumerate() {
        count[i] = mmv.lookup_value(count_metric_name, product);
        machine[i] = mmv.lookup_value(time_metric_name, product);
        inqueue[i] = mmv.lookup_value(queue_metric_name, product);
    }

    loop {
        let product = random::<usize>() % PRODUCTS_COUNT;
        let working = random::<u64>() % 3;
        thread::sleep(Duration::from_secs(working));

        mmv.inc_value(&mut machine[product], 1.0);
        count[product].inc_ull(1);

        for i in 0..PRODUCTS_COUNT {
            if i != product {
                mmv.inc_value(&mut inqueue[i], working as f64);
            }
        }
    }
}