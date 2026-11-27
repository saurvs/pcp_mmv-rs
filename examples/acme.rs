//! A port of PCP's Acme example (src/pmdas/mmv/acme.c).
//!
//! Usage: `cargo run --example acme [ITERATIONS]`
//!
//! Writes `$PCP_TMP_DIR/mmv/acme` and updates it until `ITERATIONS`
//! products have been built (forever by default).

use pcp_mmv::{
    Flags, Indom, Instance, Metric, MetricSem, MetricType, Mmv, PM_COUNT_ONE, PM_TIME_USEC, Units,
};
use std::env;
use std::thread;
use std::time::Duration;

const CLUSTER_ID: u32 = 321;
const PRODUCTS_INDOM: u32 = 61;
const PRODUCTS: [&str; 3] = ["Anvils", "Rockets", "Giant_Rubber_Bands"];

fn main() -> std::io::Result<()> {
    let iterations: Option<u64> = env::args()
        .nth(1)
        .map(|n| n.parse().expect("ITERATIONS must be a number"));

    let instances: Vec<Instance> = PRODUCTS
        .iter()
        .zip(0..)
        .map(|(product, id)| Instance::new(id, product))
        .collect();
    let indom = Indom::new(
        PRODUCTS_INDOM,
        &instances,
        "Acme products",
        "Most popular products produced by the Acme Corporation",
    );

    let microseconds = Units::new(0, 1, 0, 0, PM_TIME_USEC, 0);
    let metrics = [
        Metric::new(
            "products.count",
            7,
            MetricType::U64,
            MetricSem::Counter,
            Units::new(0, 0, 1, 0, 0, PM_COUNT_ONE),
            PRODUCTS_INDOM,
            "Acme factory product throughput",
            "Monotonic increasing counter of products produced in the Acme Corporation\n\
             factory since starting the Acme production application.  Quality guaranteed.",
        ),
        Metric::new(
            "products.time",
            8,
            MetricType::U64,
            MetricSem::Counter,
            microseconds,
            PRODUCTS_INDOM,
            "Machine time spent producing Acme products",
            "Machine time spent producing Acme Corporation products.  Does not include\n\
             time in queues waiting for production machinery.",
        ),
        Metric::new(
            "products.queuetime",
            10,
            MetricType::U64,
            MetricSem::Counter,
            microseconds,
            PRODUCTS_INDOM,
            "Queued time while producing Acme products",
            "Time spent in the queue waiting to build Acme Corporation products,\n\
             while some other Acme product was being built instead of this one.",
        ),
    ];

    let mmv = Mmv::start("acme", CLUSTER_ID, Flags::empty(), &metrics, &[indom])?;

    let lookup = |metric| {
        PRODUCTS.map(|product| {
            mmv.lookup_value(metric, Some(product))
                .expect("value exists")
        })
    };
    let count = lookup("products.count");
    let machine = lookup("products.time");
    let inqueue = lookup("products.queuetime");

    let mut built = 0;
    while iterations.is_none_or(|n| built < n) {
        // Choose a product and pretend to work on it for up to 50ms.
        let product = rand::random_range(0..PRODUCTS.len());
        let working: u64 = rand::random_range(0..50_000);
        thread::sleep(Duration::from_micros(working));

        // One more product built, using the libpcp_mmv API...
        machine[product].inc(working as f64);
        // ...or a direct update of the mapped value.
        count[product].inc_ull(1);

        // Every other product was queued for this time.
        for (i, value) in inqueue.iter().enumerate() {
            if i != product {
                value.inc(working as f64);
            }
        }
        built += 1;
    }
    Ok(())
}
