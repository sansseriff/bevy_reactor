use crate::UiBuilder;

pub trait UiTemplate {
    fn build(&self, builder: &mut UiBuilder);
}

pub trait InvokeUiTemplate {
    fn invoke<T: UiTemplate>(&mut self, template: T) -> &mut Self;
}

impl<'w> InvokeUiTemplate for UiBuilder<'w> {
    fn invoke<T: UiTemplate>(&mut self, template: T) -> &mut Self {
        template.build(self);
        self
    }
}
