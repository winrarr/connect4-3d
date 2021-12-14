#[tarpc::service]
pub trait World {
    async fn place(x: u8, y: u8, z: u8) -> String;
}
