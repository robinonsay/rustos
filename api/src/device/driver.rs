use crate::common::{Read, Write};

pub trait Driver<T, U>: Write<T> + Read<U> {
    const DRIVER_NAME: &'static str;
}
