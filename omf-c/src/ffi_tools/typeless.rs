use std::ffi::c_void;

/// An typeless container that holds a typed value.
///
/// The only action it can perform is dropping the contained value. Used to store the data
/// behind FFI pointers.
#[derive(Debug)]
pub struct Typeless {
    ptr: *mut c_void,
    drop: fn(*mut c_void),
}

impl Typeless {
    /// Moves `value` into the typeless container, returning a safe pointer to it along
    /// with the container.
    pub fn new<T: 'static>(value: T) -> (*mut T, Self) {
        let ptr: *mut T = Box::into_raw(Box::new(value));
        (
            ptr,
            Self {
                ptr: ptr.cast(),
                drop: Self::drop_func::<T>,
            },
        )
    }

    fn drop_func<T>(ptr: *mut c_void) {
        // Safety: we know where this came from so the type cast is safe.
        unsafe {
            _ = Box::from_raw(ptr.cast::<T>());
        };
    }
}

impl Drop for Typeless {
    fn drop(&mut self) {
        (self.drop)(self.ptr);
    }
}
