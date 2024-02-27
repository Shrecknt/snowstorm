use std::future::Future;

pub mod r#async;
pub mod sync;

pub trait AsyncVarint {
    type Error;

    fn write_varint(&mut self, varint: i32)
        -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn read_varint(&mut self) -> impl Future<Output = Result<i32, Self::Error>> + Send;
    fn read_varint_len(&mut self) -> impl Future<Output = Result<(u32, i32), Self::Error>> + Send;
}

pub trait SyncVarintRead {
    type Error;

    fn read_varint(&mut self) -> Result<i32, Self::Error>;
    fn read_varint_len(&mut self) -> Result<(u32, i32), Self::Error>;
}
pub trait SyncVarintWrite {
    type Error;

    fn write_varint(&mut self, varint: i32) -> Result<(), Self::Error>;
}
