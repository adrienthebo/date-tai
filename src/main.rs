extern crate chrono;
extern crate time_sys;

use errno::{errno, Errno};
use futures::sync::oneshot;
use futures::*;
use hurdles::Barrier;
use linux_api::time::timespec;
use std::thread;
use std::time::{Duration, SystemTime};

type TimeResult = Result<std::time::Duration, Errno>;

fn clock_gettime(clkid: linux_api::posix_types::clockid_t) -> TimeResult {
    let mut ts = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let rc;
    unsafe {
        rc = time_sys::clock_gettime(clkid, &mut ts);
    }

    if rc == 0 {
        Ok(std::time::Duration::new(
            ts.tv_sec as u64,
            ts.tv_nsec as u32,
        ))
    } else {
        Err(errno())
    }
}

fn get_realtime() -> TimeResult {
    clock_gettime(linux_api::time::CLOCK_TAI)
}

fn get_tai() -> TimeResult {
    clock_gettime(linux_api::time::CLOCK_REALTIME)
}

fn datetime(duration: &std::time::Duration) -> chrono::DateTime<chrono::offset::Utc> {
    let system_time = SystemTime::UNIX_EPOCH
        .checked_add(duration.clone())
        .unwrap();
    chrono::DateTime::from(system_time)
}

fn measure_par<'a, F>(clocks: Vec<F>) -> Vec<std::time::Duration>
where
    F: Fn() -> TimeResult + Sync + Send + 'static,
{
    let barrier = Barrier::new(clocks.len());
    let mut receivers = Vec::with_capacity(clocks.len());
    for clock in clocks {
        let (s, r) = oneshot::channel::<std::time::Duration>();
        receivers.push(r);

        let mut b = barrier.clone();
        thread::spawn(move || {
            b.wait();
            s.send(clock().expect("Unable to read clock")).unwrap();
        });
    }

    let results = receivers
        .into_iter()
        .map(|r| r.wait().unwrap())
        .collect::<Vec<Duration>>();
    results
}

const DATE_FMT: &'static str = "%a %b %e %T.%f %Y";

fn main() {
    let clocks: Vec<fn() -> TimeResult> = vec![get_tai, get_realtime];

    let results = measure_par(clocks.clone());
    let tai = results[0];
    let rt = results[1];

    let tai_chrono = datetime(&tai);
    let realtime_chrono = datetime(&rt);

    println!(
        "CLOCK_TAI:\t{:?}\t({})",
        &tai,
        &tai_chrono.format(DATE_FMT).to_string()
    );
    println!(
        "CLOCK_REALTIME:\t{:?}\t({})",
        &rt,
        &realtime_chrono.format(DATE_FMT).to_string()
    );

    let delta: Duration;
    let delta_sign: char;
    if rt >= tai {
        delta = rt - tai;
        delta_sign = ' ';
    } else {
        delta = tai - rt;
        delta_sign = '-';
    }

    let tai_offset = Duration::new(37, 0);
    let tai_delta: Duration;
    let tai_delta_sign: char;

    if delta >= tai_offset {
        tai_delta = delta - tai_offset;
        tai_delta_sign = ' ';
    } else {
        tai_delta = tai_offset - delta;
        tai_delta_sign = '-';
    }

    println!(
        "Delta: {delta_sign}{delta:?} (Less RT/TAI offset: {tai_delta_sign}{tai_delta:?})",
        delta_sign = delta_sign,
        delta = delta,
        tai_delta = tai_delta,
        tai_delta_sign = tai_delta_sign
    );
}
