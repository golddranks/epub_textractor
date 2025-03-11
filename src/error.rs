use std::fmt::Display;

#[must_use]
pub struct EndMsg<M> {
    pub callback: M,
    pub file: &'static str,
    pub line: u32,
}

impl<M> EndMsg<M>
where
    M: FnOnce() -> String,
{
    pub fn end_with(self, causing_err: impl Display) -> ! {
        eprintln!(
            "[{}:{}] {} at phase {}: {} {}",
            self.file,
            self.line,
            crate::EPUB_FNAME,
            crate::PHASE,
            (self.callback)(),
            causing_err
        );
        std::process::exit(2)
    }

    pub fn new(callback: M, file: &'static str, line: u32) -> Self {
        Self {
            file,
            line,
            callback,
        }
    }
}

macro_rules! 即死 {
    () => {
        crate::error::EndMsg::new(|| "".to_string(), file!(), line!()).end_with("")
    };
    ($fmt:literal $(, $args:expr)*) => {
        crate::error::EndMsg::new(|| format!($fmt, $($args),*), file!(), line!()).end_with("")
    };
}
macro_rules! 死 {
    () => {
        crate::error::EndMsg::new(|| "".to_string(), file!(), line!())
    };
    ($fmt:literal $(, $args:expr)*) => {
        crate::error::EndMsg::new(|| format!($fmt, $($args),*), file!(), line!())
    };
}
pub(crate) use 即死;
pub(crate) use 死;

pub trait OrDie<T> {
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String;
}

impl<T, E> OrDie<T> for Result<T, E>
where
    E: Display,
{
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String,
    {
        match self {
            Ok(t) => t,
            Err(e) => end_msg.end_with(e),
        }
    }
}

impl<T> OrDie<T> for Option<T> {
    fn or_<M>(self, end_msg: EndMsg<M>) -> T
    where
        M: FnOnce() -> String,
    {
        match self {
            Some(t) => t,
            None => end_msg.end_with(""),
        }
    }
}
