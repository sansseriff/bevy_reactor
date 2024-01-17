use crate::{IntoViewHandle, ViewHandle};
use impl_trait_for_tuples::*;

// ViewTuple

#[doc(hidden)]
pub trait ViewTuple {
    fn get_handles(self, out: &mut Vec<ViewHandle>);
}

impl<I: IntoViewHandle> ViewTuple for I {
    fn get_handles(self, out: &mut Vec<ViewHandle>) {
        out.push(self.into_view_handle());
    }
}

impl<I: IntoViewHandle> ViewTuple for Option<I> {
    fn get_handles(self, out: &mut Vec<ViewHandle>) {
        if let Some(view) = self {
            out.push(view.into_view_handle());
        }
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(IntoViewHandle)]
impl ViewTuple for Tuple {
    fn get_handles(self, out: &mut Vec<ViewHandle>) {
        for_tuples!(#( out.push(self.Tuple.into_view_handle()); )*)
    }
}
