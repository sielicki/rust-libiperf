#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/libiperf.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use std::ffi::{CString};
    use std::thread;
    use std::time;
    use std::sync::{Arc, Mutex, Condvar};

    fn run_iperf_client_test((host, port, m, c): (String, i32, &Mutex<bool>, &Condvar)) {
        let mut started = m.lock().unwrap();
        while !*started {
            started = c.wait(started).unwrap();
        }
        println!("Starting Client");
        std::thread::sleep(time::Duration::from_secs(1));
        unsafe {
            let t: *mut iperf_test;
            t = iperf_new_test();
            if t == std::ptr::null_mut() {
                panic!("failed to create test0");
            }
            iperf_defaults(t);
            iperf_set_test_role(t, 'c' as i8);
            iperf_set_test_server_hostname(t, CString::new(host).expect("Couldn't make cstring?").into_raw());
            iperf_set_test_server_port(t, port);
            if iperf_run_client(t) < 0 {
                panic!("Couldn't run client");
            }
        }
    }

    fn create_iperf_server_test((host, port, m, c): (String, i32, &Mutex<bool>, &Condvar)) {
        unsafe {
            let t: *mut iperf_test;
            t = iperf_new_test();
            if t == std::ptr::null_mut() {
                panic!("failed to create test0");
            }
            iperf_defaults(t);
            iperf_set_test_role(t, 's' as i8);
            iperf_set_test_server_hostname(t, CString::new(host).expect("Couldn't make cstring?").into_raw());
            iperf_set_test_server_port(t, port);

            {
                let mut started = m.lock().unwrap();
                *started = true;
                // We notify the condvar that the value has changed.
                c.notify_one();
            }
            println!("Starting Server");
            if iperf_run_server(t) < 0 {
                panic!("Couldn't run client");
            }
        }
    }

    #[test]
    fn man_page_example() {
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();
        thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            create_iperf_server_test(("127.0.0.1".to_string(), 1337, lock, cvar));
        });
        let (lock, cvar) = &*pair;
        run_iperf_client_test(("127.0.0.1".to_string(), 1337, lock, cvar)); 
    }
}