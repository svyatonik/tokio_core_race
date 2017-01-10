extern crate mio;
extern crate tokio_core;
extern crate parking_lot;
extern crate time;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::{Mutex, Condvar};

const THREADS: usize = 8;
const ITERATIONS: usize = 16;

fn main() {
    let scheduled = Arc::new(AtomicUsize::new(0));
    let restart_lock = Arc::new(Mutex::new(false));
    let restart_event = Arc::new(Condvar::new());

    // create event loop
    let poll = mio::Poll::new().unwrap();
    let (tx, rx) = tokio_core::reactor::channel();
    poll.register(&rx, mio::Token(0), mio::Ready::readable(), mio::PollOpt::edge()).unwrap();

    // spawn threads, which will send messages to single receiver
    for i in 0..THREADS {
        let scheduled = scheduled.clone();
        let tx = tx.clone();
        let restart_lock = restart_lock.clone();
        let restart_event = restart_event.clone();
        thread::spawn(move || {
            loop {
                // send `ITERATIONS` messages
                for _ in 0..ITERATIONS {
                    tx.send(i).unwrap();
                    scheduled.fetch_add(1, Ordering::SeqCst);
                }

                // now sleep some time && wake up all threads @ the same time
                let mut lock = restart_lock.lock();
                restart_event.wait_for(&mut lock, Duration::from_millis(200));
                restart_event.notify_all();
            }
        });
    }

    let mut counter: usize = 0usize;
    let mut events = mio::Events::with_capacity(1024);
    let mut proves = 0;
    loop {
        let poll_time = time::precise_time_s();
        let n = poll.poll(&mut events, Some(Duration::from_secs(1))).unwrap();
        assert!(n <= 1);

        if events.len() == 0 {
            // sometimes poll returns 0 event if it is called without timeout
            // => check that time has passed
            let now = time::precise_time_s();
            if now - poll_time >= 1f64 {
                // if we are here => receiver has stopped receiving messages
                println!("PROCESSED: {} SCHEDULED: {} PENDING: {} INCONSISTENTS: {}", counter, scheduled.load(Ordering::SeqCst), rx.pending(), rx.inconsistents());
                println!("PROVED");

                // to prove that it really has stopped receiving messages
                // => sleep couple of times
                proves += 1;
                if proves > 5 {
                    break;
                }
            } // else spurious wakeup?

            continue;
        }

        // check that we have received our event
        let event = events.get(0).unwrap();
        let token = event.token();
        assert_eq!(token, mio::Token(0));

        // read the queue until it returns Some(message)
        while let Some(_) = rx.recv().unwrap() {
            scheduled.fetch_sub(1, Ordering::SeqCst);
            counter += 1;
            // progress just to make sure it still works
            if (counter % 1000) == 0 {
                println!("PROCESSED: {} SCHEDULED: {} PENDING: {} INCONSISTENTS: {}", counter, scheduled.load(Ordering::SeqCst), rx.pending(), rx.inconsistents());
            }
        }
    }
}
