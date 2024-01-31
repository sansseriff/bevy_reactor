use crate::{IntoView, ViewRef};
use impl_trait_for_tuples::*;

// ViewTuple

#[doc(hidden)]
pub trait ViewTuple {
    fn get_handles(self, out: &mut Vec<ViewRef>);
}

impl<I: IntoView> ViewTuple for I {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        out.push(self.into_view());
    }
}

impl<I: IntoView> ViewTuple for Option<I> {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        if let Some(view) = self {
            out.push(view.into_view());
        }
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(ViewTuple)]
impl ViewTuple for Tuple {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        for_tuples!(#( self.Tuple.get_handles(out); )*)
    }
}
