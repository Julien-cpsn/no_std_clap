// Helper macros
#[macro_export]
macro_rules! value_parser {
    ($t:ty) => {
        |s: &str| -> Result<$t, ParseError> { <$t>::from_arg(s) }
    };
}

#[macro_export]
macro_rules! clap_app {
    ($name:expr) => {
        Command::new($name)
    };
    ($name:expr, $($args:expr),*) => {
        Command::new($name)$(.arg($args))*
    };
}