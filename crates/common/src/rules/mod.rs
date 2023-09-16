use auto_impl::auto_impl;
use serde::Serializer;

#[auto_impl(&,&mut,  Box, Arc, Rc, )]
pub trait Rules {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}
#[cfg(feature = "actix-web")]
impl<T: Rules> Rules for actix_web::web::Data<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <T as Rules>::serialize(self.as_ref(), serializer)
    }
}
