use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};

pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

pub trait AsyncDropConst {
    async fn async_drop_const(&self);
}

impl<T: AsyncDropConst> AsyncDrop for T {
    async fn async_drop(&mut self) {
        self.async_drop_const().await
    }
}

#[derive(Debug)]
pub struct AsyncDropWrapper<T: ?Sized + AsyncDrop> {
    dropped: Cell<bool>,
    pub data: UnsafeCell<T>,
}

impl<T: AsyncDrop> AsyncDropWrapper<T> {
    pub fn new(t: T) -> AsyncDropWrapper<T> {
        return AsyncDropWrapper {
            dropped: Cell::new(false),
            data: UnsafeCell::new(t),
        };
    }
}

impl<T: ?Sized + AsyncDrop> AsyncDropWrapper<T> {
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
        let t = unsafe { (&self.data as *const UnsafeCell<T>).read() }.into_inner();
        std::mem::forget(self);
        return t;
    }
}

impl<T: ?Sized + AsyncDrop> AsyncDropConst for AsyncDropWrapper<T> {
    // TODO: Audit this code
    async fn async_drop_const(&self) {
        if self.dropped.get() {
            panic!(
                "AsyncDropWrapper<{}> async dropped twice!",
                std::any::type_name::<T>()
            )
        }
        self.dropped.set(true);
        unsafe { self.data.get().as_mut().unwrap().async_drop().await }
    }
}

impl<T: ?Sized + AsyncDrop> Deref for AsyncDropWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl<T: ?Sized + AsyncDrop> DerefMut for AsyncDropWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}

impl<T: ?Sized + std::fmt::Display + AsyncDrop> std::fmt::Display for AsyncDropWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl<T: ?Sized + AsyncDrop> Drop for AsyncDropWrapper<T> {
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

    impl AsyncDrop for DummyAsyncDrop {
        async fn async_drop(&mut self) {
            println!("I get async dropped!")
        }
    }

    #[tokio::test]
    async fn test_func() {
        let a = DummyAsyncDrop;
        // FIXME: Can this really test???
        let a = AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(AsyncDropWrapper::new(a))))))))))))));
        a.async_drop_const().await;
        return;
    }
}