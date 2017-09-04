use super::*;

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

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(source);
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
