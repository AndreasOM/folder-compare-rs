use async_trait::async_trait;

#[async_trait]
pub trait CommandAsync: std::fmt::Debug {
    async fn run( &mut self ) -> anyhow::Result<()>;
}
