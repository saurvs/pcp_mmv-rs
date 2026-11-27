//! Watches a file with inotify (Linux only), counting how often it is
//! written and closed in `file.write_count` and publishing a hash of its
//! contents in `file.hash`.
//!
//! Usage: `cargo run --example file PATH [MAX_WRITES]`
//!
//! Writes `$PCP_TMP_DIR/mmv/file`. Stops after `MAX_WRITES` writes (never by
//! default). Editors that save by renaming a new file over the old one end
//! the watch; `nano` or `echo text > PATH` work well.

#[cfg(target_os = "linux")]
fn main() -> std::io::Result<()> {
    linux::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("this example uses inotify, which is only available on Linux");
}

#[cfg(target_os = "linux")]
mod linux {
    use inotify::{EventMask, Inotify, WatchMask};
    use pcp_mmv::{Flags, Metric, MetricSem, MetricType, Mmv, PM_COUNT_ONE, Units};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use std::{env, fs, io};

    pub fn main() -> io::Result<()> {
        let mut args = env::args().skip(1);
        let path = args.next().expect("usage: file PATH [MAX_WRITES]");
        let max_writes: Option<u64> = args
            .next()
            .map(|n| n.parse().expect("MAX_WRITES must be a number"));

        let metrics = [
            Metric::new(
                "file.hash",
                1,
                MetricType::U64,
                MetricSem::Instant,
                Units::default(),
                0,
                "Hash of the file's contents",
                "Uses the default hasher of std::collections::HashMap",
            ),
            Metric::new(
                "file.write_count",
                2,
                MetricType::U64,
                MetricSem::Counter,
                Units::new(0, 0, 1, 0, 0, PM_COUNT_ONE),
                0,
                "Write count",
                "Number of times the file was written to and closed",
            ),
        ];
        let mmv = Mmv::start("file", 1, Flags::empty(), &metrics, &[])?;
        let hash = mmv.lookup_value("file.hash", None).expect("value exists");
        let writes = mmv
            .lookup_value("file.write_count", None)
            .expect("value exists");

        let mut inotify = Inotify::init()?;
        inotify.watches().add(&path, WatchMask::CLOSE_WRITE)?;
        println!("Watching {path}");

        let mut buffer = [0; 4096];
        while max_writes.is_none_or(|n| writes.ull() < n) {
            for event in inotify.read_events_blocking(&mut buffer)? {
                if event.mask.contains(EventMask::CLOSE_WRITE) {
                    let mut hasher = DefaultHasher::new();
                    hasher.write(&fs::read(&path)?);
                    hash.set_ull(hasher.finish());
                    writes.inc_ull(1);
                    println!("CLOSE_WRITE event received");
                }
            }
        }
        Ok(())
    }
}
