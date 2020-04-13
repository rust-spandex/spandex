use crate::layout::pages::Page;

#[derive(Debug)]
pub struct Error {
    page: Page,
}

#[derive(Debug)]
pub enum ConfigError {
    PageWithoutColumn(Error),
}
