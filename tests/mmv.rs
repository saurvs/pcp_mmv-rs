//! Integration tests against the system `libpcp_mmv`.

use pcp_mmv::{
    Flags, Indom, Instance, Metric, MetricSem, MetricType, Mmv, PM_COUNT_ONE, PM_TIME_USEC, Units,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

/// An MMV file path in a fresh temporary directory, removed on drop.
struct TempMmv(PathBuf);

impl TempMmv {
    fn new(name: &str) -> TempMmv {
        let dir = std::env::temp_dir().join(format!("pcp_mmv-test-{}-{name}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        TempMmv(dir.join(name))
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }

    fn contents(&self) -> Vec<u8> {
        fs::read(&self.0).unwrap()
    }
}

impl Drop for TempMmv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

fn counter(name: &str, item: u32, indom: u32) -> Metric {
    Metric::new(
        name,
        item,
        MetricType::U64,
        MetricSem::Counter,
        Units::new(0, 0, 1, 0, 0, PM_COUNT_ONE),
        indom,
        "short help",
        "long help",
    )
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn writes_mmv_v1_header() {
    let file = TempMmv::new("header");
    let mmv = Mmv::start(file.path(), 42, Flags::empty(), &[counter("a", 1, 0)], &[]).unwrap();
    let bytes = file.contents();
    assert_eq!(&bytes[0..4], b"MMV\0");
    assert_eq!(u32::from_ne_bytes(bytes[4..8].try_into().unwrap()), 1);
    // The two generation numbers match once the file is ready.
    assert_eq!(bytes[8..16], bytes[16..24]);
    // Cluster ID.
    assert_eq!(u32::from_ne_bytes(bytes[36..40].try_into().unwrap()), 42);
    assert!(contains(&bytes, b"short help\0"));
    assert!(contains(&bytes, b"long help\0"));
    drop(mmv);
    // Without Flags::PROCESS the file outlives the handle.
    assert!(file.0.exists());
}

#[test]
fn updates_singular_values() {
    let file = TempMmv::new("singular");
    let metrics = [
        counter("requests", 1, 0),
        Metric::new(
            "latency",
            2,
            MetricType::Double,
            MetricSem::Instant,
            Units::new(0, 1, 0, 0, PM_TIME_USEC, 0),
            0,
            "",
            "",
        ),
        Metric::new(
            "version",
            3,
            MetricType::String,
            MetricSem::Discrete,
            Units::default(),
            0,
            "",
            "",
        ),
    ];
    let mmv = Mmv::start(file.path(), 1, Flags::empty(), &metrics, &[]).unwrap();

    assert!(mmv.lookup_value("missing", None).is_none());

    let requests = mmv.lookup_value("requests", None).unwrap();
    assert_eq!(requests.ull(), 0);
    requests.inc(2.0);
    requests.inc_ull(3);
    assert_eq!(requests.ull(), 5);
    requests.set(10.0);
    assert_eq!(requests.ull(), 10);
    requests.set_ull(7);
    // Another lookup sees the same value.
    assert_eq!(mmv.lookup_value("requests", None).unwrap().ull(), 7);

    let latency = mmv.lookup_value("latency", None).unwrap();
    latency.set(1.5);
    latency.inc(1.0);
    assert_eq!(f64::from_bits(latency.ull()), 2.5);

    let version = mmv.lookup_value("version", None).unwrap();
    version.set_string("v1.2.3");
    assert!(contains(&file.contents(), b"v1.2.3\0"));
    // Too long for libpcp_mmv: ignored.
    version.set_string(&"x".repeat(300));
    assert!(contains(&file.contents(), b"v1.2.3\0"));
}

#[test]
fn updates_instance_values() {
    let file = TempMmv::new("instances");
    let instances = [Instance::new(0, "red"), Instance::new(1, "blue")];
    let indom = Indom::new(5, &instances, "Colours", "Some colours");
    let mmv = Mmv::start(
        file.path(),
        1,
        Flags::empty(),
        &[counter("paint", 1, 5)],
        &[indom],
    )
    .unwrap();

    let red = mmv.lookup_value("paint", Some("red")).unwrap();
    let blue = mmv.lookup_value("paint", Some("blue")).unwrap();
    red.inc(1.0);
    blue.inc(2.0);
    assert_eq!((red.ull(), blue.ull()), (1, 2));

    assert!(mmv.lookup_value("paint", Some("green")).is_none());
    assert!(mmv.lookup_value("paint", None).is_none());
    assert!(contains(&file.contents(), b"blue\0"));
}

#[test]
fn process_flag_removes_file_on_drop() {
    let file = TempMmv::new("process");
    let mmv = Mmv::start(file.path(), 1, Flags::PROCESS, &[counter("a", 1, 0)], &[]).unwrap();
    assert!(file.0.exists());
    drop(mmv);
    assert!(!file.0.exists());
}

#[test]
fn reports_errors() {
    let file = TempMmv::new("errors");

    let long_help = Indom::new(5, &[Instance::new(0, "a")], &"x".repeat(256), "");
    let err = Mmv::start(
        file.path(),
        1,
        Flags::empty(),
        &[counter("a", 1, 5)],
        &[long_help],
    )
    .unwrap_err();
    assert_eq!(err.raw_os_error(), Some(7), "E2BIG: {err}");

    let missing_indom = counter("a", 1, 99);
    let err = Mmv::start(file.path(), 1, Flags::empty(), &[missing_indom], &[]).unwrap_err();
    assert_eq!(err.raw_os_error(), Some(3), "ESRCH: {err}");

    let err = Mmv::start(
        file.path(),
        4096,
        Flags::empty(),
        &[counter("a", 1, 0)],
        &[],
    )
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}

#[test]
#[should_panic(expected = "longer than 63 bytes")]
fn rejects_long_metric_names() {
    counter(&"x".repeat(64), 1, 0);
}

#[test]
fn accepts_63_byte_metric_names() {
    let file = TempMmv::new("long-name");
    let name = "x".repeat(63);
    let mmv = Mmv::start(file.path(), 1, Flags::empty(), &[counter(&name, 1, 0)], &[]).unwrap();
    assert!(mmv.lookup_value(&name, None).is_some());
}
