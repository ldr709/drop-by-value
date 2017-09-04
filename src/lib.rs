#![warn(missing_docs)]

#[allow(unused_imports)]
#[macro_use]
extern crate drop_by_value_derive;

use std::mem;
use mem::ManuallyDrop;
use std::ptr;
use std::cmp::Ordering;
use std::ops::Deref;
use std::ops::DerefMut;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct DropRef<T: Destructure<R>, R>(ManuallyDrop<T>, PhantomData<R>);

impl<T: Destructure<R>, R> DropRef<T, R> {
    pub fn into_inner(mut self) -> R {
        let x = unsafe { drop_ref_mut_to_inner(&mut self) };
        mem::forget(self);
        x
    }
}

impl<T: Destructure<R>, R> Drop for DropRef<T, R> {
    fn drop(&mut self) {
        // Destructure and drop. Don't call T's drop, as this type will be passed into T's drop and
        // so that would cause infinite recursion.
        unsafe { drop_ref_mut_to_inner(self) };
    }
}

impl<T: Destructure<R>, R> Deref for DropRef<T, R> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Destructure<R>, R> DerefMut for DropRef<T, R> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

// Do not use this module, as it is not actually public. It has to be pub so macros can use it.
#[doc(hidden)]
pub mod internal {
    use super::*;

    pub trait Destructure<Result> {
        fn destructure(Self) -> Result;
    }

    // Like with ptr::read, t should not be used afterwards.
    unsafe fn extract_unsafe<T>(t: &mut ManuallyDrop<T>) -> T {
        ptr::read(t.deref_mut())
    }

    // Ditto.
    pub unsafe fn drop_ref_mut_to_inner<T: Destructure<R>, R>(self_: &mut DropRef<T, R>) -> R {
        Destructure::destructure(extract_unsafe(&mut self_.0))
    }

    pub fn drop_ref_new<T: Destructure<R>, R>(t: T) -> DropRef<T, R> {
        DropRef(ManuallyDrop::new(t), PhantomData)
    }

    // Make sure that the argument impls Destructure.
    pub fn make_sure_destructure<T: Destructure<R>, R>(_: &T) {}
}

use internal::*;

pub trait DropValue<Result>: Sized + Destructure<Result> {
    fn drop_value(self_: DropRef<Self, Result>);
}

mod drop_wrapper;
pub use drop_wrapper::DropByValueWrapper;

// Allow internal use of macros.
mod drop_by_value {
    pub use super::*;
}

// Destructure a DropByValue type into its inner type.
#[macro_export]
macro_rules! destructure {
    ($exp:expr) => {{
        let x = $exp;

        // Make sure we have visibility.
        &x.0;
        ::drop_by_value::internal::make_sure_destructure(&x);
        ::drop_by_value::internal::make_sure_destructure(&x.0);

        ::drop_by_value::internal::Destructure::destructure(x)
    }};
}

mod drop_guard;
pub use drop_guard::*;
