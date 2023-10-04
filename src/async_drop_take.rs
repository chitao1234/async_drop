use std::cell::{Cell, UnsafeCell};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub trait AsyncDropTake {
    async fn async_drop(self);
}

// struct AsyncDropGuard<'a, T: AsyncDrop> {
//     wrapper: &'a AsyncDropWrapper<T>,
// }

#[derive(Debug)]
pub struct AsyncDropWrapper<T: AsyncDropTake> {
    dropped: Cell<bool>,
    pub data: UnsafeCell<ManuallyDrop<T>>,
}

impl<T: AsyncDropTake> AsyncDropWrapper<T> {
    pub fn new(t: T) -> AsyncDropWrapper<T> {
        return AsyncDropWrapper {
            dropped: Cell::new(false),
            data: UnsafeCell::new(ManuallyDrop::new(t)),
        };
    }
}

impl<T: AsyncDropTake> AsyncDropWrapper<T> {
    pub fn borrow(&self) -> &T {
        unsafe { self.data.get().as_ref().unwrap() }
    }

    pub fn borrow_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    // TODO: Audit this code
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        ManuallyDrop::into_inner(unsafe { self.data.get().read() })
    }
}

impl<T: AsyncDropTake> AsyncDropTake for AsyncDropWrapper<T> {
    // TODO: Audit this code
    async fn async_drop(self) {
        if self.dropped.get() {
            panic!(
                "AsyncDropWrapper<{}> async dropped twice!",
                std::any::type_name::<T>()
            )
        }
        self.dropped.set(true);
        ManuallyDrop::into_inner(unsafe { self.data.get().read() })
            .async_drop()
            .await
    }
}

impl<T: AsyncDropTake> Deref for AsyncDropWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl<T: AsyncDropTake> DerefMut for AsyncDropWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}

impl<T: std::fmt::Display + AsyncDropTake> std::fmt::Display for AsyncDropWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl<T: AsyncDropTake> Drop for AsyncDropWrapper<T> {
    fn drop(&mut self) {
        if !self.dropped.get() {
            panic!(
                "AsyncDropWrapper<{}> not async dropped before drop!",
                std::any::type_name::<T>()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyAsyncDrop;

    impl AsyncDropTake for DummyAsyncDrop {
        async fn async_drop(self) {
            println!("I get async dropped!")
        }
    }

    #[tokio::test]
    async fn test_func() {
        let a = DummyAsyncDrop;
        // FIXME: Can this really test???
        let a = AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(a))))))))))))));
        a.async_drop().await;
        return;
    }
}
