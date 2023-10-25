use std::{
    ffi::c_char,
    panic::{RefUnwindSafe, UnwindSafe},
    ptr::{null, null_mut},
};

use super::{catch, typeless::Typeless};

/// A trait that defines how an object can be moved into a pointer and then freed later.
///
/// This is similar to the built-in `From` with a matching `IntoFfi` trait.
pub trait FfiConvert<T>
where
    Self: Sized + RefUnwindSafe + 'static,
    T: UnwindSafe + 'static,
{
    /// Turns an object into a pointer, putting any required allocations into `storage`.
    fn convert(value: T, storage: &mut FfiStorage) -> Self;
}

/// Provides the `into_ffi` method on the wrapped type, retuning a pointer to the new
/// wrapper. Call `ffi_from_free` to free the returned pointer.
///
/// Don't implement this directly, implement `FfiConvert` or `FfiWrapper` instead.
pub trait IntoFfi<T>: Sized + UnwindSafe + 'static
where
    T: FfiConvert<Self>,
{
    fn into_ffi(self) -> *mut T;
}

impl<T, U> IntoFfi<U> for T
where
    U: FfiConvert<T>,
    T: UnwindSafe + 'static,
{
    /// Calls `FfiFrom::convert` on the matching implementation.
    #[inline]
    fn into_ffi(self) -> *mut U {
        catch::panic(|| {
            let mut storage = FfiStorage::new();
            let wrapper: U = FfiConvert::convert(self, &mut storage);
            Box::into_raw(Box::new(WrapperAndStorage { wrapper, storage })).cast()
        })
        .unwrap_or_else(null_mut)
    }
}

/// This trait declares that a type is an FFI wrapper for `T`, adding an easy way to
/// implement `FfiFrom` for simple wrappers.
pub trait FfiWrapper<T>
where
    Self: Sized + RefUnwindSafe + 'static,
    T: UnwindSafe + 'static,
{
    /// Create the wrapper object, which may safely contain pointers into boxes and vecs
    /// inside `value`.
    fn wrap(value: &T) -> Self;
}

impl<U: FfiWrapper<T>, T> FfiConvert<T> for U
where
    T: UnwindSafe + 'static,
{
    fn convert(value: T, storage: &mut FfiStorage) -> Self {
        let wrapper = Self::wrap(&value);
        storage.keep(value);
        wrapper
    }
}

/// Stores arbitrary data needed by an FFI wrapper.
#[derive(Default, Debug)]
pub struct FfiStorage {
    data: Vec<Typeless>,
}

impl FfiStorage {
    pub fn new() -> Self {
        Default::default()
    }

    /// Keep an object alive, returing a pointer to it.
    pub fn keep<T: UnwindSafe + 'static>(&mut self, value: T) -> *const T {
        let (ptr, typeless) = Typeless::new(value);
        self.data.push(typeless);
        ptr.cast_const()
    }

    /// Keep an object alive, returing a pointer to it.
    pub fn keep_mut<T: UnwindSafe + 'static>(&mut self, value: T) -> *mut T {
        let (ptr, typeless) = Typeless::new(value);
        self.data.push(typeless);
        ptr
    }

    /// Keep a vec alive, returing a pointer to the contents.
    pub fn keep_vec<T: UnwindSafe + 'static>(&mut self, values: Vec<T>) -> *const T {
        let ptr = values.as_ptr();
        self.keep(values);
        ptr
    }

    /// Nul-terminate a string, keep it alive, and return the `const char*` pointer.
    ///
    /// Returns null if the string is empty.
    pub fn keep_string(&mut self, input: impl Into<String>) -> *const c_char {
        let mut bytes: Vec<u8> = input.into().into_bytes();
        if bytes.is_empty() {
            null()
        } else {
            bytes.push(0);
            self.keep_vec(bytes).cast()
        }
    }

    /// Convert an object, keeping its depdendent data in here as well.
    pub fn convert<U, T>(&mut self, value: T) -> U
    where
        U: FfiConvert<T> + UnwindSafe + 'static,
        T: UnwindSafe + 'static,
    {
        U::convert(value, self)
    }

    pub fn convert_ptr<U, T>(&mut self, value: T) -> *const U
    where
        U: FfiConvert<T> + UnwindSafe + 'static,
        T: UnwindSafe + 'static,
    {
        let wrapper = self.convert(value);
        self.keep(wrapper)
    }

    pub fn convert_ptr_mut<U, T>(&mut self, value: T) -> *mut U
    where
        U: FfiConvert<T> + UnwindSafe + 'static,
        T: UnwindSafe + 'static,
    {
        let wrapper = self.convert(value);
        self.keep_mut(wrapper)
    }

    /// Convert an optional object, keeping its depdendent data in here as well,
    /// returning a pointer or null.
    pub fn convert_option<U, T>(&mut self, value: Option<T>) -> *const U
    where
        U: FfiConvert<T> + UnwindSafe + 'static,
        T: UnwindSafe + 'static,
    {
        if let Some(val) = value {
            self.convert_ptr(val)
        } else {
            null()
        }
    }

    pub fn convert_iter_term<U, T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> (*const U, usize)
    where
        U: FfiConvert<T> + Default + UnwindSafe + 'static,
        T: UnwindSafe + 'static,
    {
        let vec: Vec<_> = values
            .into_iter()
            .map(|x| self.convert(x))
            .chain(Some(Default::default()))
            .collect();
        let len = vec.len() - 1;
        let ptr = self.keep_vec(vec);
        (ptr, len)
    }
}

/// Frees a pointer created by `IntoFfi::into_ffi`.
///
/// Does nothing if the pointer is null.
///
/// # Safety
///
/// `ptr` must have been returned by `IntoFfi::into_ffi` and have not been modified.
pub unsafe fn into_ffi_free<T: RefUnwindSafe>(ptr: *mut T) -> bool {
    catch::panic_bool(|| {
        if !ptr.is_null() {
            let ws_ptr: *mut WrapperAndStorage<T> = ptr.cast();
            unsafe {
                _ = Box::from_raw(ws_ptr);
            }
        }
    })
}

#[allow(dead_code)]
#[repr(C)]
struct WrapperAndStorage<W> {
    wrapper: W,
    storage: FfiStorage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_storage_keep() {
        let mut s = FfiStorage::new();
        let x = 42;
        let y = s.keep(x);
        assert_eq!(unsafe { *y }, 42);
        drop(s);
    }

    #[test]
    fn ffi_storage_keep_vec() {
        let mut s = FfiStorage::new();
        let x = vec![42, 43, 44];
        let y = s.keep_vec(x);
        assert_eq!(unsafe { *y }, 42);
        assert_eq!(unsafe { *y.add(1) }, 43);
        assert_eq!(unsafe { *y.add(2) }, 44);
        drop(s);
    }

    #[test]
    fn ffi_storage_keep_string() {
        let mut s = FfiStorage::new();
        let ptr = s.keep_string("test string");
        let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), 12) };
        assert_eq!(slice, b"test string\0");
        drop(s);
    }

    #[test]
    #[allow(dead_code)]
    fn ffi_into_and_free() {
        struct Test {
            inner: String,
        }
        struct TestWrapper {
            inner: *const c_char,
        }
        impl FfiConvert<Test> for TestWrapper {
            fn convert(value: Test, storage: &mut FfiStorage) -> Self {
                Self {
                    inner: storage.keep_string(value.inner),
                }
            }
        }
        let test = Test {
            inner: "Test string".to_owned(),
        };
        let wrapper: *mut TestWrapper = test.into_ffi();
        assert!(!wrapper.is_null());
        unsafe { into_ffi_free(wrapper) };
    }
}
