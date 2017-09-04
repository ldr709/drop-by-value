use super::*;

#[derive(Default, Clone, DropByValue)]
#[DropByValue(name = "DropGuard", vis = "pub", derive(Default))]
struct DropGuardImpl<F: FnOnce()>(F);

impl<F: FnOnce()> DropValue<DropGuardImpl<F>> for DropGuard<F> {
    fn drop_value(self_: DropRef<Self, DropGuardImpl<F>>) {
        (self_.into_inner().0)();
    }
}

impl<F: FnOnce()> DropGuard<F> {
    pub fn new(f: F) -> Self {
        DropGuard(DropGuardImpl(f).into())
    }

    pub fn into_inner(self) -> F {
        destructure!(self).0
    }
}

impl<F: FnOnce()> Deref for DropGuard<F> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.0 .0
    }
}

impl<F: FnOnce()> DerefMut for DropGuard<F> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.0 .0
    }
}

impl<F: FnOnce()> From<F> for DropGuard<F> {
    fn from(f: F) -> Self {
        DropGuard::new(f)
    }
}

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
