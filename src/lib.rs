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

// Cannot do Copy since ManuallyDrop doesn't derive it.
#[derive(Debug)]
pub struct DropByValueWrapper<T>(ManuallyDrop<T>);

impl<T> DropByValueWrapper<T> {
    fn new(t: T) -> Self {
        DropByValueWrapper(ManuallyDrop::new(t))
    }
}

impl<T> From<T> for DropByValueWrapper<T> {
    fn from(t: T) -> Self {
        DropByValueWrapper::new(t)
    }
}

impl<T> Deref for DropByValueWrapper<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T> DerefMut for DropByValueWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

impl<T> Destructure<T> for DropByValueWrapper<T> {
    fn destructure(self_: Self) -> T {
        ManuallyDrop::into_inner(self_.0)
    }
}

// impl everything implable and derivable.
impl<T: Clone> Clone for DropByValueWrapper<T> {
    fn clone(&self) -> Self {
        self.0.clone().into()
    }
}

impl<T: Default> Default for DropByValueWrapper<T> {
    fn default() -> Self {
        DropByValueWrapper::new(Default::default())
    }
}

impl<T: PartialEq> PartialEq for DropByValueWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.0.ne(other)
    }
}

impl<T: Eq> Eq for DropByValueWrapper<T> {}

impl<T: PartialOrd> PartialOrd for DropByValueWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }

    fn lt(&self, other: &Self) -> bool {
        self.0.lt(other)
    }

    fn le(&self, other: &Self) -> bool {
        self.0.le(other)
    }

    fn gt(&self, other: &Self) -> bool {
        self.0.gt(other)
    }

    fn ge(&self, other: &Self) -> bool {
        self.0.ge(other)
    }
}

impl<T: Ord> Ord for DropByValueWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(other)
    }
}

impl<T: Hash> Hash for DropByValueWrapper<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, DropByValue)]
    #[DropByValue(name = "A", vis = "pub", derive(Default))]
    struct AImpl<F>
    where
        F: FnOnce(),
    {
        f: F,
    }

    impl<F: FnOnce()> DropValue<AImpl<F>> for A<F> {
        fn drop_value(self_: DropRef<Self, AImpl<F>>) {
            (self_.into_inner().f)();
        }
    }

    impl<F: FnOnce()> A<F> {
        pub fn new(f: F) -> Self {
            A(AImpl { f: f }.into())
        }

        pub fn get(&mut self) -> &mut F {
            &mut self.0.f
        }

        pub fn extract(self) -> F {
            destructure!(self).f
        }
    }

    #[test]
    fn test_a() {
        let mut x: u32 = 0;
        let z: u32 = 0xdeadbeef;
        let y = Box::<u32>::new(z);

        assert!(x != z);
        {
            let x_ref = &mut x;
            let f = move || *x_ref = *y;
            let a = A::new(f);

            let f = a.extract();
            A::new(f);
        }
        assert_eq!(x, z);
    }

    #[test]
    fn test_get() {
        let mut x: u32 = 0;

        {
            let mut f = || x += 1;
            f();

            let mut a = A::new(f);
            a.get()();
        }

        assert_eq!(x, 3);
    }
}
