use async_trait::async_trait;

#[async_trait]
pub trait Node {
    async fn init(&mut self) -> ();
    async fn run(&mut self) -> ();
}
