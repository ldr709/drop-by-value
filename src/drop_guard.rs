use super::*;

drop_move_wrap! {
    /// Run a [`FnOnce`] function on drop.
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
    /// Construct from a [`FnOnce`] function.
    pub fn new(f: F) -> Self {
        DropGuardInner { func: f }.into()
    }

    /// Extract the function.
    pub fn into_inner(self) -> F {
        let inner = DropGuardInner::from(self);
        inner.func
    }
}

impl<F: FnOnce()> Deref for DropGuard<F> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.0.func
    }
}

impl<F: FnOnce()> DerefMut for DropGuard<F> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.0.func
    }
}

impl<F: FnOnce()> From<F> for DropGuard<F> {
    fn from(f: F) -> Self {
        DropGuard::new(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate std;
    use std::boxed::Box;

    #[test]
    fn test_drop() {
        let mut x: u32 = 0;
        let z: u32 = 0xdeadbeef;
        let y = Box::<u32>::new(z);

        assert!(x != z);
        {
            let x_ref = &mut x;
            let f = move || *x_ref = *y;
            let guard = DropGuard::new(f);

            let f = guard.into_inner();
            DropGuard::new(f);
        }
        assert_eq!(x, z);
    }

    #[test]
    fn test_get() {
        let mut x: u32 = 0;

        {
            let mut f = || x += 1;
            f();

            let mut guard = DropGuard::new(f);
            guard.deref_mut()();
        }

        assert_eq!(x, 3);
    }
}
