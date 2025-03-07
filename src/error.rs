macro_rules! 死 {
    ($ctx:expr, $error:ident) => {{
        eprintln!("[{}:{}] {} at phase {}: {}", file!(), line!(), $ctx.epub_fname, $ctx.phase, $error);
        std::process::exit(2);
    }};
    ($ctx:expr, $fmt:literal $(, $args:expr)*) => {{
        eprintln!("[{}:{}] {} at phase {}: {}", file!(), line!(), $ctx.epub_fname, $ctx.phase, format!($fmt, $($args),*));
        std::process::exit(2);
    }};
}
pub(crate) use 死;

pub trait ResultOrDie<T, E> {
    fn or_die(self, f: impl FnOnce(E)) -> T; // TODO: impl FnOnce(E) -> ! once that gets stable
}

impl<T, E> ResultOrDie<T, E> for Result<T, E> {
    fn or_die(self, f: impl FnOnce(E)) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                f(e);
                unreachable!()
            }
        }
    }
}

pub trait OptionOrDie<T> {
    fn or_die(self, f: impl FnOnce()) -> T; // TODO: impl FnOnce() -> ! once that gets stable
}

impl<T> OptionOrDie<T> for Option<T> {
    fn or_die(self, f: impl FnOnce()) -> T {
        match self {
            Some(t) => t,
            None => {
                f();
                unreachable!()
            }
        }
    }
}
