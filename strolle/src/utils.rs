use std::time::Instant;

pub fn measure<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let tt = Instant::now();
    let result = f();

    metric(name, tt);

    result
}

#[cfg(feature = "metrics")]
pub fn metric(metric: &str, tt: Instant) {
    use std::env;
    use std::sync::OnceLock;
    use std::time::Duration;

    use log::trace;

    static METRIC_THRESHOLD: OnceLock<Duration> = OnceLock::new();

    let threshold = METRIC_THRESHOLD.get_or_init(|| {
        env::var("STROLLE_METRIC_THRESHOLD")
            .ok()
            .map(|threshold| humantime::parse_duration(&threshold).unwrap())
            .unwrap_or_else(|| Duration::from_millis(0))
    });

    let tt = tt.elapsed();

    if tt > *threshold {
        trace!("metric({metric})={tt:?}");
    }
}

#[cfg(not(feature = "metrics"))]
pub fn metric(_metric: &str, _tt: Instant) {
    //
}
