use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "success.stpl")]
pub struct SuccessPage {}

impl SuccessPage {
    pub fn new() -> SuccessPage {
        SuccessPage {}
    }
}

#[derive(TemplateOnce)]
#[template(path = "error.stpl")]
pub struct ErrorPage<'a> {
    pub server_name: &'a str,
}

impl<'a> ErrorPage<'a> {
    pub fn new(server_name: &'a str) -> ErrorPage {
        ErrorPage { server_name }
    }
}
