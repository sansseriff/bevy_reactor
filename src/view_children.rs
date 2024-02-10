use crate::{Fragment, ViewHandle};
use impl_trait_for_tuples::*;

/// A tuple of [`View`]s which can be converted into a [`Vec<ViewHandle>`].
#[doc(hidden)]
pub trait ViewChildren {
    #[doc(hidden)]
    fn get_handles(self, out: &mut Vec<ViewHandle>);

    fn to_vec(self) -> Vec<ViewHandle>;

    /// Convert this tuple of views into a [`ViewHandle`] containing a [`Fragment`].
    fn fragment(self) -> ViewHandle;
}

impl<I: Into<ViewHandle>> ViewChildren for I {
    fn get_handles(self, out: &mut Vec<ViewHandle>) {
        out.push(self.into());
    }

    fn to_vec(self) -> Vec<ViewHandle> {
        let mut out = Vec::new();
        self.get_handles(&mut out);
        out
    }

    fn fragment(self) -> ViewHandle {
        ViewHandle::new(Fragment::new(self))
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(ViewChildren)]
impl ViewChildren for Tuple {
    fn get_handles(self, out: &mut Vec<ViewHandle>) {
        for_tuples!(#( self.Tuple.get_handles(out); )*)
    }

    fn to_vec(self) -> Vec<ViewHandle> {
        let mut out = Vec::new();
        self.get_handles(&mut out);
        out
    }

    fn fragment(self) -> ViewHandle {
        ViewHandle::new(Fragment::new(self))
    }
}
