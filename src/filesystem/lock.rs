use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use core::hint::spin_loop;
use core::ops::{Drop, Deref, DerefMut};
use core::cell::UnsafeCell;

//Simple semaphor based Read Write Lock
pub struct RWLock<T: ?Sized>
{
    rlock: AtomicUsize,
    wlock: AtomicBool,
    data: UnsafeCell<T>,
}

// Guards are returned when Lock is Locked
pub struct RLockGuard<'a, T: ?Sized + 'a>
{
    rlock: &'a AtomicUsize,
    data: &'a mut T,
}

pub struct WLockGuard<'a, T: ?Sized + 'a>
{
    wlock: &'a AtomicBool,
    data: &'a mut T,
}

// When dereferenced the Locks return the underlaying data
impl<'a, T: ?Sized> Deref for RLockGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}


impl<'a, T: ?Sized> DerefMut for RLockGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {

        &mut *self.data }
}

impl<'a, T: ?Sized> Deref for WLockGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for WLockGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {

        &mut *self.data }
}


impl<'a, T: ?Sized> Drop for RLockGuard<'a, T>
{
    /// The dropping of the ReadGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.rlock.fetch_sub(1, Ordering::SeqCst);
    }

}

impl<'a, T: ?Sized> Drop for WLockGuard<'a, T>
{
    /// The dropping of the WriteGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.wlock.store(false,Ordering::SeqCst);
    }

}

unsafe impl<T: ?Sized + Send> Sync for RWLock<T> {}
unsafe impl<T: ?Sized + Send> Send for RWLock<T> {}

impl<T> RWLock<T>{
    // New lock has reads = 0 and writes = false
    pub const fn new(user_data: T) -> RWLock<T>
    {
        RWLock
        {
            wlock: AtomicBool::new(false),
            rlock: AtomicUsize::new(0),
            data: UnsafeCell::new(user_data),
        }
    }
}


impl<T: ?Sized> RWLock<T>
{
    // Internal function to obtain write lock
    fn obtain_wlock(&self)
    {
        // Loop as long as the lock is true
        while self.wlock.compare_and_swap(false, true, Ordering::SeqCst) != false
        {
            // Wait until the lock looks unlocked before retrying
            while self.wlock.load(Ordering::SeqCst)
            {
                spin_loop()
            }
        }

        // When the write Lock is obtained wait until all reads have finished
        while self.rlock.load(Ordering::SeqCst) != 0
        {

            while self.rlock.load(Ordering::SeqCst) != 0
            {
                spin_loop()
            }
        }
    }

    // Internal function to obtain read lock
    fn obtain_rlock(&self)
    {
        let mut success = false;
        while !success {
            // wait till write lock is false
            while self.wlock.load(Ordering::SeqCst) == true
            {
                while self.wlock.load(Ordering::SeqCst)
                {
                    spin_loop()
                }
            }

            // increment the read semaphore
            self.rlock.fetch_add(1, Ordering::SeqCst);
            success = true;

            // make sure no write locks have occured in the mean time.
            if self.wlock.load(Ordering::SeqCst) == true
            {
                self.rlock.fetch_sub(1, Ordering::SeqCst);
                success = false;
            }

        }
    }

    // public read lock method.
    pub fn rlock(&self) -> RLockGuard<T>
    {
        self.obtain_rlock();
        RLockGuard
        {
            rlock: &self.rlock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    //public write lock method.
    pub fn wlock(&self) -> WLockGuard<T>
    {
        self.obtain_wlock();
        WLockGuard
        {
            wlock: &self.wlock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}