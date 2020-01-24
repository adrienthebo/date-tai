extern crate time_sys;
extern crate time;

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

fn main() {
    let tai = get_tai();
    let realtime = get_realtime();
    println!("CLOCK_TAI: {:?}", &tai);
    println!("CLOCK_REALTIME: {:?}", &realtime);

    if let (Ok(r), Ok(t)) = (realtime, tai) {
    let delta = r - t;
        println!("Delta: {:?}", delta);
    }
}
