use super::*;

/// A wrapper around [`T::Outer`](DropMoveTypes::Outer) that will drop by converting to `T` and
/// dropping, rather than dropping the `T::Outer`.
///
/// The trouble with making `drop` be pass by move is that the `self` parameter may end up being
/// dropped, creating infinite recursion. To avoid this, a `DropHandle` is passed to
/// [`drop_move`](DropMove::drop_move) so that if it goes out of scope it will not create infinite
/// recursion. This implements [`Deref`] and [`DerefMut`] for easy access to functions on
/// `T::Outer`.
#[derive(Debug)]
pub struct DropHandle<T: DropMoveTypes>(ManuallyDrop<T::Outer>);

impl<T: DropMoveTypes> DropHandle<T> {
    unsafe fn take(self_: &mut Self) -> T::Outer {
        ManuallyDrop::take(&mut self_.0)
    }

    /// Convert to the inner structure `T`.
    ///
    /// This is an associated function so that will not conflict with any methods of `T::Outer`,
    /// which are accessible through [`Deref`].
    pub fn into_inner(self_: Self) -> T {
        Self::into_outer(self_).into()
    }

    /// Convert to the outer structure `T::Outer`.
    ///
    /// Be careful when using this function. It is easy to end up recursively calling `drop` on the
    /// output by accident, creating an infinite recursive loop.
    ///
    /// This is an associated function so that will not conflict with any methods of `T::Outer`,
    /// which are accessible through [`Deref`].
    pub fn into_outer(mut self_: Self) -> T::Outer {
        let outer = unsafe { Self::take(&mut self_) };
        mem::forget(self_);
        outer
    }
}

impl<T: DropMoveTypes> From<T> for DropHandle<T> {
    fn from(t: T) -> Self {
        Self(ManuallyDrop::new(t.into()))
    }
}

impl<T: DropMoveTypes> Drop for DropHandle<T> {
    fn drop(&mut self) {
        let _inner: T = unsafe { Self::take(self) }.into();

        // Dropping the inner type avoids the drop calling drop infinite loop.
    }
}

impl<T: DropMoveTypes> Deref for DropHandle<T> {
    type Target = T::Outer;
    fn deref(&self) -> &T::Outer {
        self.0.deref()
    }
}

impl<T: DropMoveTypes> DerefMut for DropHandle<T> {
    fn deref_mut(&mut self) -> &mut T::Outer {
        self.0.deref_mut()
    }
}
