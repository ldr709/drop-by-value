# drop-move

    This crate allows implementing [`Drop`] as "pass by move". Here is an example of how this can be
    used to call a [`FnOnce`] from [`drop`].

    ```
    use drop_move::{drop_move_wrap, DropMove, DropHandle};

    drop_move_wrap! {
        /// Run a function on drop.
        #[derive(Clone)]
        pub struct DropGuard<F: FnOnce()>(DropGuardInner {
            func: F,
        });
    }

    impl<F: FnOnce()> DropMove for DropGuardInner<F> {
        fn drop_move(self_: DropHandle<Self>) {
            (DropHandle::into_inner(self_).func)()
        }
    }

    impl<F: FnOnce()> DropGuard<F> {
        pub fn new(f: F) -> Self {
            DropGuardInner { func: f }.into()
        }
    }

    let mut x: u32 = 0;
    {
        let y = Box::new(&mut x); // Box is not Copy, so the closure will only be FnOnce.
        let guard = DropGuard::new(move || **y += 1);
    }

    assert_eq!(x, 1);
    ```

    By implementing the [`DropMove`] trait, we were able to have `func` run when the `DropGuard`
    goes out of scope. The usual [`Drop`] trait only allows `drop(&mut self)`, which does not allow
    moving the members of the `DropGuard`, as is required to call a [`FnOnce`]. The reason they do
    not allow `drop(self)` is that it would be too easy to accidentally end up dropping `self`,
    leading to an infinite loop. According to Rust's usual semantics `self` would be dropped at the
    end of the scope, and even if a special case were added for `drop(self)` it could still easily
    happen in a function called by `drop`.

    These problems are mostly avoided by wrapping `self` in a [`DropHandle`], which will only drop
    each member of the structure when it goes out of scope, rather than calling `drop` recursively.
    Semantically, [`drop_move`](DropMove::drop_move) can be thought of as destructuring `DropGuard`.
    Each unmoved member will be dropped when it goes out of scope. These members can be accessed by
    value through [`into_inner`](DropHandle::into_inner). The original `DropGuard` can be obtained
    from [`into_outer`](DropHandle::into_outer), but you must be careful to avoid infinite recursion
    when using this.

    Given this destructuring viewpoint, it should be no surprise that `drop_move` also supports
    destructuring, which is normally not allowed for types that implement [`Drop`]. Here, we can
    convert the `DropGuard` back into the function it contains.

    ```
  # use drop_move::{drop_move_wrap, DropMove, DropHandle};
  #
  # drop_move_wrap! {
  #     #[derive(Clone)]
  #     pub struct DropGuard<F: FnOnce()>(DropGuardInner {
  #         func: F,
  #     });
  # }
  #
  # impl<F: FnOnce()> DropMove for DropGuardInner<F> {
  #     fn drop_move(self_: DropHandle<Self>) {
  #         (DropHandle::into_inner(self_).func)()
  #     }
  # }
    impl<F: FnOnce()> DropGuard<F> {
        /// Extract the function.
        pub fn into_inner(self) -> F {
            let inner: DropGuardInner<F> = self.into();
            inner.func
        }
    }
    ```

    How this works is that [`drop_move_wrap!`] expands into two structure definitions.

    ```ignore
  # use drop_move::DropMoveWrapper;
    /// Run a function on drop.
    #[derive(Clone)]
    pub struct DropGuard<F: FnOnce()>(DropMoveWrapper<DropGuardInner<F>>);

    /// Run a function on drop.
    #[derive(Clone)]
    struct DropGuardInner<F: FnOnce()> {
        func: F,
    };
    ```

    The outer structure `DropGuard` provides the public interface, while `DropGuardInner` contains
    the actual members of the `struct`. Neither will implement [`Drop`]. Instead, DropMoveWrapper
    will implement [`Drop`] based on the [`DropMove`] you provide. The structure members can be
    borrowed from a `DropGaurd` using `&self.0.func`, because [`DropMoveWrapper`]
    implements [`Deref`]. They can moved by converting the `DropGaurd` to a `DropGuardInner` with
    [`DropMoveWrapper::into_inner(self.0)`](DropMoveWrapper::into_inner) or
    [`self.into()`](Into::into).

    Notice that the doc comments and attributes have been duplicated for both structures. In fact,
    doc comments are treated as [attributes](https://stackoverflow.com/a/33999625/4071916) by the
    compiler.

    The macro also creates a few trait implementations.

    ```
  #  use drop_move::{DropMove, DropMoveTypes, DropMoveWrapper, DropHandle};
  # /// Run a function on drop.
  # #[derive(Clone)]
  # pub struct DropGuard<F: FnOnce()>(DropMoveWrapper<DropGuardInner<F>>);
  #
  # /// Run a function on drop.
  # #[derive(Clone)]
  # struct DropGuardInner<F: FnOnce()> {
  #     func: F,
  # };
  #
    impl<F: FnOnce()> From<DropGuard<F>> for DropGuardInner<F> {
        fn from(x: DropGuard<F>) -> Self {
            DropMoveWrapper::into_inner(x.0)
        }
    }

    impl<F: FnOnce()> From<DropGuardInner<F>> for DropGuard<F> {
        fn from(x: DropGuardInner<F>) -> Self {
            Self(DropMoveWrapper::new(x))
        }
    }

    impl<F: FnOnce()> DropMoveTypes for DropGuardInner<F>
    {
        type Outer = DropGuard<F>;
    }
  #
  # impl<F: FnOnce()> DropMove for DropGuardInner<F> {
  #     fn drop_move(self_: DropHandle<Self>) {
  #         (DropHandle::into_inner(self_).func)()
  #     }
  # }
  #
  # impl<F: FnOnce()> DropGuard<F> {
  #     /// Construct from an existing function.
  #     pub fn new(f: F) -> Self {
  #         DropGuard(DropMoveWrapper::new(DropGuardInner { func: f }))
  #     }
  # }
  #
  # let mut x: u32 = 0;
  # {
  #     let y = Box::new(&mut x);
  #     // Box is not Copy, so the closure will only be FnOnce.
  #     let guard = DropGuard::new(move || **y += 1);
  # }
  #
  # assert_eq!(x, 1);
    ```

    The implementation of [`DropMoveTypes`] lets [`DropMoveWrapper`] and [`DropHandle`] know the
    relationship between `DropGuard` and `DropGuardInner`. It is implemented on the inner structure
    because this will keep the implementation private in the common case that the inner structure is
    private but the outer is public. The [`From`] implementations are so that they know how to
    convert back and forth, and also function as convenience methods for creating and destructuring
    `DropGuard`s.

    You may be wondering why `drop_move` takes a [`DropHandle`] rather than passing in the inner
    structure `DropGuardInner`, which would behave correctly for destructuring and would drop the
    members individually without calling `drop` again when it goes out of scope. However, you
    wouldn't easily be able to call a `&self` or `&mut self` function, which would want an instance
    of `DropGuard` instead. It would require reconstructing the [`DropHandle`] again so that it can
    be borrowed, then carefully destructuring it after the call to avoid infinite `drop` recursion.
    [`DropHandle`] allows you to avoid this error prone construction by implementing [`Deref`] for
    the outer structure.

    See [`drop_move_wrap!`] for its full supported syntax. See the source for [`DropGuard`] for the
    full example.
