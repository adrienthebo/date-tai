extern crate time_sys;
extern crate time;
extern crate chrono;

use std::time::SystemTime;
use errno::{Errno, errno};
use linux_api::time::timespec;

fn clock_gettime(clkid: linux_api::posix_types::clockid_t) -> Result<time::Timespec, Errno> {
    let mut ts = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let tp = &mut ts;
    let rc;
    unsafe {
        rc = time_sys::clock_gettime(clkid, tp);
    }

    if rc == 0 {
        Ok(time::Timespec { sec: ts.tv_sec, nsec: ts.tv_nsec as i32})
    } else {
        Err(errno())
    }
}

fn get_realtime() -> Result<time::Timespec, Errno> {
    clock_gettime(linux_api::time::CLOCK_TAI)
}

fn get_tai() -> Result<time::Timespec, Errno> {
    clock_gettime(linux_api::time::CLOCK_REALTIME)
}

fn chrono(ts: time::Timespec) -> chrono::DateTime<chrono::offset::Utc> {
    let dur = time::Duration::nanoseconds(ts.nsec as i64) + time::Duration::seconds(ts.sec);
    let system_time = SystemTime::UNIX_EPOCH.checked_add(dur.to_std().unwrap()).unwrap();
    chrono::DateTime::from(system_time)
}

fn main() {
    let tai_ts = get_tai();
    let realtime_ts = get_realtime();

    let tai_chrono = tai_ts.clone().ok().map(|ts| chrono(ts));
    let realtime_chrono = realtime_ts.clone().ok().map(|ts| chrono(ts));



    println!("CLOCK_TAI:\t{:?}\t({:?})", &tai_ts, &tai_chrono);
    println!("CLOCK_REALTIME:\t{:?}\t({:?})", &realtime_ts, &realtime_chrono);

    if let (Ok(r), Ok(t)) = (realtime_ts, tai_ts) {
    let delta = r - t;
        println!("Delta: {:?}", delta);
    }
}
