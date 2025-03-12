use std::{fmt::Display, sync::Mutex};

#[derive(Debug)]
pub struct GlobalStr {
    str: Mutex<String>,
}

impl GlobalStr {
    pub const fn new() -> Self {
        GlobalStr {
            str: Mutex::new(String::new()),
        }
    }

    pub fn get(&self) -> String {
        self.str.lock().expect("shouldn't be poisoned").clone()
    }

    pub fn set(&self, str: impl Into<String>) {
        *self.str.lock().expect("shouldn't be poisoned") = str.into();
    }
}

impl Display for GlobalStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.str.lock().expect("shouldn't be poisoned");
        f.write_str(&str)
    }
}
