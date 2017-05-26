extern crate inotify;
extern crate pcp_mmv;
extern crate sha2;

use std::env;
use inotify::*;
use pcp_mmv::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::fs::File;
use std::io::prelude::*;

/*
 * usage: ./file <path-to-monitor>
 */

fn main() {

    /*
     * create hash metric
     */

    let mut count_dimension = PMUnits::new();
    count_dimension.set_dimCount(1);
    count_dimension.set_scaleCount(PM_COUNT_ONE as i32);
    let file_hash_metric = Metric::new(
        "file.hash", // name
        7, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
        count_dimension,
        0,
        "Hash of the file's contents",
        "Uses the default hasher for std::collections::hash_map"
    );

    /*
     * create write count metric
     */

    let write_count_metric = Metric::new(
        "file.write_count", // name
        7, // item
        MetricType::MMV_TYPE_U64,
        MetricSem::MMV_SEM_COUNTER,
        count_dimension,
        0,
        "Write count",
        "Number of times the file was written to and closed"
    );

    /*
     * start mmv
     */

    let metrics = [file_hash_metric, write_count_metric];
    let mut mmv = MMV::new();
    mmv.start("file", 1, 0, &metrics, &[]);

    /*
     * setup inotify
     */

    let mut args = env::args();
    let path = args.nth(1).unwrap();
    let mut inotify = Inotify::init().unwrap();
    inotify.add_watch(&path, watch_mask::CLOSE_WRITE).unwrap();

    println!("Watching {}", path);

    let mut file_hash_val = mmv.lookup_value("file.hash", "");
    let mut write_count_val = mmv.lookup_value("file.write_count", "");

    /*
     * respond to file writes
     */
            
    let mut event_buffer = [0u8; 4096];
    loop {
        let events = inotify
            .read_events_blocking(&mut event_buffer).unwrap();

        for event in events {
            if event.mask.contains(event_mask::CLOSE_WRITE) {
                let mut file = File::open(&path).unwrap();
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();

                let mut hasher = DefaultHasher::new();
                hasher.write(&contents);
                file_hash_val.set_ull(hasher.finish());

                write_count_val.inc_ull(1);

                println!("CLOSE_WRITE event recieved");  
            }
        }
    }
}