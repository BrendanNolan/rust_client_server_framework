use auto_impl::auto_impl;

#[auto_impl(&, Box, Arc)]
pub trait RequestProcessor<Req, Resp> {
    fn process(&self, request: &Req) -> Resp;
}
