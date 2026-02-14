use core::cell::UnsafeCell;

use mutex_traits::{ConstInit, ScopedRawMutex};

pub struct ScopedLocked<Mutex: ScopedRawMutex + ConstInit, T> {
    mutex: Mutex,
    data: UnsafeCell<T>,
}

unsafe impl<Mutex: ScopedRawMutex + ConstInit + Sync, T: Send> Sync for ScopedLocked<Mutex, T> {}

impl<Mutex: ScopedRawMutex + ConstInit, T> ScopedLocked<Mutex, T> {
    pub const fn new(data: T) -> Self {
        Self {
            mutex: Mutex::INIT,
            data: UnsafeCell::new(data),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.mutex.with_lock(|| {
            let value = unsafe { &mut *self.data.get() };
            f(value)
        })
    }

    pub fn try_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.mutex.try_with_lock(|| {
            let value = unsafe { &mut *self.data.get() };
            f(value)
        })
    }
}

#[cfg(test)]
pub(crate) mod test_mutex {
    use core::{
        panic,
        sync::atomic::{AtomicBool, Ordering::SeqCst},
    };
    use mutex::ConstInit;
    use mutex_traits::ScopedRawMutex;

    pub(crate) struct Mutex(std::sync::Mutex<AtomicBool>);

    unsafe impl ScopedRawMutex for Mutex {
        fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
            if let Ok(lock) = self.0.try_lock() {
                if lock.swap(false, SeqCst) {
                    let res = Some(f());
                    lock.store(true, SeqCst);
                    res
                } else {
                    None
                }
            } else {
                None
            }
        }

        fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
            let lock = self.0.lock().unwrap();
            if lock.swap(false, SeqCst) {
                let res = f();
                lock.store(true, SeqCst);
                res
            } else {
                panic!("Already locked");
            }
        }

        fn is_locked(&self) -> bool {
            if let Ok(lock) = self.0.try_lock() {
                lock.load(SeqCst)
            } else {
                true
            }
        }
    }

    impl ConstInit for Mutex {
        const INIT: Self = Self(std::sync::Mutex::new(AtomicBool::new(true)));
    }

}
