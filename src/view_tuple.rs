use crate::{IntoView, ViewRef};
use impl_trait_for_tuples::*;

// ViewTuple

#[doc(hidden)]
pub trait ViewTuple {
    fn get_handles(self, out: &mut Vec<ViewRef>);

    fn to_vec(self) -> Vec<ViewRef>;
}

impl<I: IntoView> ViewTuple for I {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        out.push(self.into_view());
    }

    fn to_vec(self) -> Vec<ViewRef> {
        let mut out = Vec::new();
        self.get_handles(&mut out);
        out
    }
}

impl<I: IntoView> ViewTuple for Option<I> {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        if let Some(view) = self {
            out.push(view.into_view());
        }
    }

    fn to_vec(self) -> Vec<ViewRef> {
        let mut out = Vec::new();
        self.get_handles(&mut out);
        out
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(ViewTuple)]
impl ViewTuple for Tuple {
    fn get_handles(self, out: &mut Vec<ViewRef>) {
        for_tuples!(#( self.Tuple.get_handles(out); )*)
    }

    fn to_vec(self) -> Vec<ViewRef> {
        let mut out = Vec::new();
        self.get_handles(&mut out);
        out
    }
}
