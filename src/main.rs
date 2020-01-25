extern crate time_sys;
extern crate chrono;

use std::time::SystemTime;
use errno::{Errno, errno};
use linux_api::time::timespec;

fn clock_gettime(clkid: linux_api::posix_types::clockid_t) -> Result<std::time::Duration, Errno> {
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
        Ok(std::time::Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    } else {
        Err(errno())
    }
}

fn get_realtime() -> Result<std::time::Duration, Errno> {
    clock_gettime(linux_api::time::CLOCK_TAI)
}

fn get_tai() -> Result<std::time::Duration, Errno> {
    clock_gettime(linux_api::time::CLOCK_REALTIME)
}

fn datetime(duration: &std::time::Duration) -> chrono::DateTime<chrono::offset::Utc> {
    let system_time = SystemTime::UNIX_EPOCH.checked_add(duration.clone()).unwrap();
    chrono::DateTime::from(system_time)
}

const DATE_FMT: &'static str = "%a %b %e %T.%f %Y";

fn main() {
    let tai = get_tai().expect("Unable to read CLOCK_TAI");
    let rt = get_realtime().expect("Unable to read CLOCK_REALTIME");

    let tai_chrono = datetime(&tai);
    let realtime_chrono = datetime(&rt);

    println!("CLOCK_TAI:\t{:?}\t({})", &tai, &tai_chrono.format(DATE_FMT).to_string());
    println!("CLOCK_REALTIME:\t{:?}\t({})", &rt, &realtime_chrono.format(DATE_FMT).to_string());
    println!("Delta: {:?}", rt - tai);
}
