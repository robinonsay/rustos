use crate::common::{Read, Write};

pub trait Driver<T>: Write<T> + Read<T>
{
    const DRIVER_NAME: &'static str; 
}
