extern crate pcp_mmv;
#[macro_use] extern crate nickel;

use pcp_mmv::*;
use nickel::Nickel;

fn main() {
    
    /*
     * create metric
     */

    let mut count_dimension = PMUnits::new();
    count_dimension.set_dimCount(1);
    count_dimension.set_scaleCount(PM_COUNT_ONE as i32);
    let http_get_metric = Metric::new(
        "http.get", // name
        7, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
        count_dimension,
        0,
        "HTTP GET requests",
        "The number of HTTP GET requests recieved on the server."
    );

    /*
     * start mmv
     */

    let metrics = [http_get_metric];
    let mut mmv = MMV::new();
    mmv.start("http", 1, 0, &metrics, &[]);

    /*
     * start server
     */

    let mut server = Nickel::new();
    server.utilize(router! {
        get "**" => |_req, _res| {
            let mut http_get_val = mmv.lookup_value("http.get", "");
            mmv.inc_value(&mut http_get_val, 1.0);

            "Your GET request was instrumented!"
        }
    });
    server.listen("127.0.0.1:6767").unwrap();
}