tokio-core/src/reactor/channel.rs is slightly changed just to illustrate the race:
```Rust
use std::sync::atomic::{AtomicUsize, Ordering};
...
pub struct Receiver<T> {
    inconsistents: AtomicUsize,
    ctl: ReceiverCtl,
...
    let rx = Receiver {
        inconsistents: AtomicUsize::new(0),
        ctl: rx,
...
impl<T> Receiver<T> {
    pub fn pending(&self) -> usize {
        self.ctl.pending()
    }

    pub fn inconsistents(&self) -> usize {
        self.inconsistents.load(Ordering::SeqCst)
    }

    pub fn recv(&self) -> io::Result<Option<T>> {
...
            // inconsistent state.
            PopResult::Inconsistent => {
                self.inconsistents.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            }
            PopResult::Empty  => Ok(None),

```

mio/src/channel.rs is slightly changed just to illustrate the race:
```Rust
...
impl ReceiverCtl {
    pub fn pending(&self) -> usize {
        self.inner.pending.load(Ordering::Acquire)
    }

    pub fn dec(&self) -> io::Result<()> {
...
```

Example output:
```sh
PROCESSED: 1000 SCHEDULED: 72 PENDING: 72 INCONSISTENTS: 0
PROCESSED: 2000 SCHEDULED: 270 PENDING: 271 INCONSISTENTS: 0
PROCESSED: 3000 SCHEDULED: 40 PENDING: 40 INCONSISTENTS: 0
PROCESSED: 4000 SCHEDULED: 353 PENDING: 353 INCONSISTENTS: 0
PROCESSED: 5000 SCHEDULED: 232 PENDING: 232 INCONSISTENTS: 0
PROCESSED: 6000 SCHEDULED: 256 PENDING: 256 INCONSISTENTS: 0
PROCESSED: 7000 SCHEDULED: 33 PENDING: 34 INCONSISTENTS: 0
PROCESSED: 8000 SCHEDULED: 240 PENDING: 240 INCONSISTENTS: 0
PROCESSED: 9000 SCHEDULED: 79 PENDING: 80 INCONSISTENTS: 0
PROCESSED: 10000 SCHEDULED: 256 PENDING: 256 INCONSISTENTS: 0
PROCESSED: 11000 SCHEDULED: 712 PENDING: 712 INCONSISTENTS: 0
PROCESSED: 12000 SCHEDULED: 992 PENDING: 992 INCONSISTENTS: 0
PROCESSED: 13000 SCHEDULED: 264 PENDING: 264 INCONSISTENTS: 0
PROCESSED: 14000 SCHEDULED: 139 PENDING: 140 INCONSISTENTS: 0
PROCESSED: 15000 SCHEDULED: 120 PENDING: 120 INCONSISTENTS: 0
PROCESSED: 16000 SCHEDULED: 201 PENDING: 203 INCONSISTENTS: 0
PROCESSED: 17000 SCHEDULED: 120 PENDING: 120 INCONSISTENTS: 0
PROCESSED: 17970 SCHEDULED: 41726 PENDING: 41726 INCONSISTENTS: 1
PROVED
PROCESSED: 17970 SCHEDULED: 52478 PENDING: 52478 INCONSISTENTS: 1
PROVED
PROCESSED: 17970 SCHEDULED: 59470 PENDING: 59470 INCONSISTENTS: 1
PROVED
PROCESSED: 17970 SCHEDULED: 69614 PENDING: 69614 INCONSISTENTS: 1
PROVED
PROCESSED: 17970 SCHEDULED: 77374 PENDING: 77374 INCONSISTENTS: 1
PROVED
PROCESSED: 17970 SCHEDULED: 83646 PENDING: 83646 INCONSISTENTS: 1
PROVED
```
