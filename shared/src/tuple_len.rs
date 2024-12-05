// The TupleLen trait allows compile-time checking of the length of a tuple.
// This is useful for statically determining the number of columns in a diesel
// schema table.
//
// Use it like this:
//
// let num_columns = orm::schema::(table_name)::all_columns.len();
//
// If you need to support tuples with more than 12 elements, you can add more
// type parameters to the tuple! macro invocation at the bottom of this file.
//

pub trait TupleLen {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Base case for empty tuple
impl TupleLen for () {
    fn len(&self) -> usize {
        0
    }
}

macro_rules! peel {
    ($name:ident,) => {};
    ($first:ident, $($rest:ident,)+) => {
        tuple! { $($rest,)+ }
    };
}

macro_rules! tuple {
    () => {};
    ( $($name:ident,)+ ) => {
        impl<$($name),+> TupleLen for ($($name,)+) {
            #[inline]
            fn len(&self) -> usize {
                count_idents!($($name),+)
            }
        }
        peel! { $($name,)+ }
    }
}

macro_rules! count_idents {
    () => { 0 };
    ($name:ident) => { 1 };
    ($first:ident, $($rest:ident),+) => { 1 + count_idents!($($rest),+) };
}

// Initial invocation with maximum number of type parameters
tuple! { L, K, J, I, H, G, F, E, D, C, B, A, }
