#[macro_export]
macro_rules! default_arguments {
    ($var:tt) => {
        let $var = $var.unwrap_or_default();
    };

    ($($var:tt),*) => {
        $(default_arguments!($var);)*
    }
}
