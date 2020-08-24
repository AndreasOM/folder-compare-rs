use async_trait::async_trait;

#[async_trait]
pub trait CommandAsync {
    async fn run( &mut self ) -> anyhow::Result<()>;
}
