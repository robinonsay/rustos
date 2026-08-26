pub trait ErrorType {
    type Error: core::fmt::Debug;
}

pub trait Write<T>: ErrorType {
    fn write(&mut self, value: T) -> Result<(), Self::Error>;
}

pub trait Read<T>: ErrorType {
    fn read(&mut self) -> Result<T, Self::Error>;
}
