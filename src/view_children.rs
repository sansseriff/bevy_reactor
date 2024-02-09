use crate::ViewHandle;
use impl_trait_for_tuples::*;

#[doc(hidden)]
pub trait ViewChildren {
    fn get_handles(self, out: &mut Vec<ViewHandle>);

    fn to_vec(self) -> Vec<ViewHandle>;
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
}
