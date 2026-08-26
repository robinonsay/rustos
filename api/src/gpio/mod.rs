use crate::common::{Read, Write};

pub trait GpioPin: Write<bool> + Read<bool> {}
