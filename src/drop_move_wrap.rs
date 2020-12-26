use super::*;

/// A wrapper around the inner structure `T` that calls `drop_move` when it is dropped.
///
/// The inner structure members can be borrowed using the [`Deref`] and [`DerefMut`]
/// implementations, or be moved with `into_inner`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DropMoveWrapper<T: DropMove>(ManuallyDrop<T>);

impl<T: DropMove> DropMoveWrapper<T> {
    unsafe fn take(self_: &mut Self) -> T {
        ManuallyDrop::take(&mut self_.0)
    }

    /// Wrap the inner structure, so that it will be dropped with `drop_move`.
    pub fn new(x: T) -> Self {
        DropMoveWrapper(ManuallyDrop::new(x))
    }

    /// Convert into the inner structure `T`.
    ///
    /// This is an associated function so that will not conflict with any methods of the inner type,
    /// which are accessible through [`Deref`].
    pub fn into_inner(mut self_: Self) -> T {
        let inner = unsafe { Self::take(&mut self_) };
        mem::forget(self_);
        inner
    }
}

impl<T: DropMove> Deref for DropMoveWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: DropMove> DerefMut for DropMoveWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<T: DropMove> Drop for DropMoveWrapper<T> {
    fn drop(&mut self) {
        let drop_ref: DropHandle<T> = From::from(unsafe { Self::take(self) });
        DropMove::drop_move(drop_ref);
    }
}

/** Generate a pair of structures to allow moving out of `drop`.

    The syntax is roughly:
    ```ignore
    #[shared_attributes]
    {
        #[outer_only_attributes]
    }
    outer_visibility struct outer_name<...>(
        #[inner_only_attributes] inner_visibility inner_structure {
            members
        }
    ) where ...;
    ```

    Tuple `structs` can be used by swapping `{ members }` for `( members )`, and enumerations by
    changing `struct` to `enum`. The attributes, generic parameters, and where clause are optional
    and can be omitted. The syntax for the generic parameters and bounds is almost the same as
    normal; however, due to
    [limitations](https://internals.rust-lang.org/t/allow-to-follow-path-fragments-in-declarative-macros/13676)
    in macro parsing they do not support the `+` syntax for specifying multiple traits. Instead, you
    should use `:`, so e.g. `T: Clone : Eq` means that `T` must implement both `Clone` and `Eq`.

    The macro expands to two structures: `struct outer_name` wrapping a [`DropMoveWrapper`]
    containing `struct inner_name`, which holds the actual members. All attributes in
    `shared_attributes` are applied to both. Doc comments
    [are also](https://stackoverflow.com/a/33999625/4071916) attributes, and can also be used here.
    The inner visibility is applied to both the definition of the inner `struct` and the field of
    the outer `struct` that wraps it, so if it is `pub` then anyone will be able to access it.

    This macro also implements [`From`] to convert back and forth between the inner and outer
    structures, and [`DropMoveTypes`] to tell [`DropMoveWrapper`] the relationship between the inner
    and outer structures.

    Note that this macro is implemented internally using a few others, which may appear in compiler
    error messages. These all have names prefixed with `drop_move_wrap`.
 */
#[macro_export]
macro_rules! drop_move_wrap {
    {$($def:tt)+} => {
        $crate::drop_move_wrap_match!{$($def)+}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! drop_move_wrap_match {
    // struct/enum {}
    {
        $(#[$attrs:meta])*
        $({$(#[$outer_only_attrs:meta])+})?
        $vis:vis struct $name:ident $(<
            $($lifetimes:lifetime $(: $lifetime_bounds1:lifetime $(+ $lifetime_bounds2:lifetime)*)?),*
            $(,)?
            $($types:ident $(:
                $($lifetime_ty_bounds1:lifetime)? $($type_bounds1:path)?
                $(: $($lifetime_ty_bounds2:lifetime)? $($type_bounds2:path)?)*
            )?),*
            $(,)?
        >)?(
            $(#[$inner_only_attrs:meta])*
            $inner_vis:vis $inner_name:ident {$($members:tt)*}
        ) $(where
            $(
                $($lifetime_wheres:lifetime)?
                $($(for<($for_lt:lifetime),*>)? $type_wheres:ty)?
                :
                $($lifetime_ty_bounds3:lifetime)? $($type_bounds3:path)?
                $(: $($lifetime_ty_bounds4:lifetime)? $($type_bounds4:path)?)*
            ),*
            $(,)?
        )?;
    } => {
        $crate::drop_move_wrap_transcribe!{
            { $(#[$attrs])* $($(#[$outer_only_attrs])+)? },
            { $(#[$attrs])* $(#[$inner_only_attrs])* },
            $vis, $inner_vis,
            struct,
            $name, $inner_name,
            { $(<$($lifetimes, )*$($types, )*>)? },
            { $(<
                $($lifetimes $(: $lifetime_bounds1 $(+ $lifetime_bounds2)*)?, )*
                $($types $(:
                    $($type_bounds1)? $($lifetime_ty_bounds1)?
                    $(+ $($type_bounds2)? $($lifetime_ty_bounds2)?)*
                )?, )*
            >)? },
            { $(where
                $(
                    $($lifetime_wheres)?
                    $($(for<($for_lt),*>)? $type_wheres)?
                    :
                    $($type_bounds3)? $($lifetime_ty_bounds3)?
                    $(+ $($type_bounds4)? $($lifetime_ty_bounds4)?)*
                ,)*
            )? },
            { $($members)* },
        }
    };

    // struct ()
    {
        $(#[$attrs:meta])*
        $({$(#[$outer_only_attrs:meta])+})?
        $vis:vis struct $name:ident $(<
            $($lifetimes:lifetime $(: $lifetime_bounds1:lifetime $(+ $lifetime_bounds2:lifetime)*)?),*
            $(,)?
            $($types:ident $(:
                $($lifetime_ty_bounds1:lifetime)? $($type_bounds1:path)?
                $(: $($lifetime_ty_bounds2:lifetime)? $($type_bounds2:path)?)*
            )?),*
            $(,)?
        >)?(
            $(#[$inner_only_attrs:meta])*
            $inner_vis:vis $inner_name:ident ($($members:tt)*)
        ) $(where
            $(
                $($lifetime_wheres:lifetime)?
                $($(for<($for_lt:lifetime),*>)? $type_wheres:ty)?
                :
                $($lifetime_ty_bounds3:lifetime)? $($type_bounds3:path)?
                $(: $($lifetime_ty_bounds4:lifetime)? $($type_bounds4:path)?)*
            ),*
            $(,)?
        )?;
    } => {
        $crate::drop_move_wrap_transcribe!{
            { $(#[$attrs])* $($(#[$outer_only_attrs])+)? },
            { $(#[$attrs])* $(#[$inner_only_attrs])* },
            $vis, $inner_vis,
            tuple,
            $name, $inner_name,
            { $(<$($lifetimes, )*$($types, )*>)? },
            { $(<
                $($lifetimes $(: $lifetime_bounds1 $(+ $lifetime_bounds2)*)?, )*
                $($types $(:
                    $($type_bounds1)? $($lifetime_ty_bounds1)?
                    $(+ $($type_bounds2)? $($lifetime_ty_bounds2)?)*
                )?, )*
            >)? },
            { $(where
                $(
                    $($lifetime_wheres)?
                    $($(for<($for_lt),*>)? $type_wheres)?
                    :
                    $($type_bounds3)? $($lifetime_ty_bounds3)?
                    $(+ $($type_bounds4)? $($lifetime_ty_bounds4)?)*
                ,)*
            )? },
            { $($members)* },
        }
    };

    // enum
    {
        $(#[$attrs:meta])*
        $({$(#[$outer_only_attrs:meta])+})?
        $vis:vis enum $name:ident $(<
            $($lifetimes:lifetime $(: $lifetime_bounds1:lifetime $(+ $lifetime_bounds2:lifetime)*)?),*
            $(,)?
            $($types:ident $(:
                $($lifetime_ty_bounds1:lifetime)? $($type_bounds1:path)?
                $(: $($lifetime_ty_bounds2:lifetime)? $($type_bounds2:path)?)*
            )?),*
            $(,)?
        >)?(
            $(#[$inner_only_attrs:meta])*
            $inner_vis:vis $inner_name:ident {$($members:tt)*}
        ) $(where
            $(
                $($lifetime_wheres:lifetime)?
                $($(for<($for_lt:lifetime),*>)? $type_wheres:ty)?
                :
                $($lifetime_ty_bounds3:lifetime)? $($type_bounds3:path)?
                $(: $($lifetime_ty_bounds4:lifetime)? $($type_bounds4:path)?)*
            ),*
            $(,)?
        )?;
    } => {
        $crate::drop_move_wrap_transcribe!{
            { $(#[$attrs])* $($(#[$outer_only_attrs])+)? },
            { $(#[$attrs])* $(#[$inner_only_attrs])* },
            $vis, $inner_vis,
            enum,
            $name, $inner_name,
            { $(<$($lifetimes, )*$($types, )*>)? },
            { $(<
                $($lifetimes $(: $lifetime_bounds1 $(+ $lifetime_bounds2)*)?, )*
                $($types $(:
                    $($type_bounds1)? $($lifetime_ty_bounds1)?
                    $(+ $($type_bounds2)? $($lifetime_ty_bounds2)?)*
                )?, )*
            >)? },
            { $(where
                $(
                    $($lifetime_wheres)?
                    $($(for<($for_lt),*>)? $type_wheres)?
                    :
                    $($type_bounds3)? $($lifetime_ty_bounds3)?
                    $(+ $($type_bounds4)? $($lifetime_ty_bounds4)?)*
                ,)*
            )? },
            { $($members)* },
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! drop_move_wrap_transcribe {
    {
        { $($attrs:tt)* },
        { $($inner_attrs:tt)* },
        $vis:vis, $inner_vis:vis,
        $decl_kind:ident,
        $name:ident, $inner_name:ident,
        { $($generic_params:tt)* },
        { $($generic_bounds:tt)* },
        { $($where_clause:tt)* },
        { $($members:tt)* }$(,)?
    } => {
        $($attrs)*
        $vis struct $name$($generic_bounds)*(
            $inner_vis $crate::DropMoveWrapper<$inner_name$($generic_params)*>
        ) $($where_clause)*;

        $crate::drop_move_wrap_inner_decl!{
            { $($inner_attrs)* },
            $inner_vis, $decl_kind,
            { $inner_name$($generic_bounds)* },
            { $($where_clause)* },
            { $($members)* },
        }

        impl$($generic_bounds)* From<$name$($generic_params)*> for $inner_name$($generic_params)*
        $($where_clause)* {
            fn from(x: $name$($generic_params)*) -> Self {
                $crate::DropMoveWrapper::into_inner(x.0)
            }
        }

        impl$($generic_bounds)* From<$inner_name$($generic_params)*> for $name$($generic_params)*
        $($where_clause)* {
            fn from(x: $inner_name$($generic_params)*) -> Self {
                Self($crate::DropMoveWrapper::new(x))
            }
        }

        impl$($generic_bounds)* $crate::DropMoveTypes for $inner_name$($generic_params)*
        $($where_clause)* {
            type Outer = $name$($generic_params)*;
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! drop_move_wrap_inner_decl {
    {
        { $($inner_attrs:tt)* },
        $inner_vis:vis, struct,
        { $($inner_type:tt)* },
        { $($where_clause:tt)* },
        { $($members:tt)* },
    } => {
        $($inner_attrs)*
        $inner_vis struct $($inner_type)* $($where_clause)* { $($members)* }
    };

    {
        { $($inner_attrs:tt)* },
        $inner_vis:vis, tuple,
        { $($inner_type:tt)* },
        { $($where_clause:tt)* },
        { $($members:tt)* },
    } => {
        $($inner_attrs)*
        $inner_vis struct $($inner_type)* ( $($members)* ) $($where_clause)*;
    };

    {
        { $($inner_attrs:tt)* },
        $inner_vis:vis, enum,
        { $($inner_type:tt)* },
        { $($where_clause:tt)* },
        { $($members:tt)* },
    } => {
        $($inner_attrs)*
        $inner_vis enum $($inner_type)* $($where_clause)* { $($members)* }
    };
}
