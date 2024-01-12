#[macro_export]
macro_rules! default_args {
    ($var:tt) => {
        let $var = $var.unwrap_or_default();
    };

    ($($var:tt),*) => {
        $(default_args!($var);)*
    }
}
