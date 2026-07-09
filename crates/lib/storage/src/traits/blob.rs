pub trait Blob {
    fn key(&self) -> String;
    fn bytes(&self) -> Vec<u8>;
}