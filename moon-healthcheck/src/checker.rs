use async_trait::async_trait;
use super::state::Protocol;
use uuid::Uuid;

#[async_trait]
pub trait Checker : Send {
    async fn check(&mut self);
    fn is_healthy(&self) -> bool;
    fn get_uuid(&self) -> Uuid;
    fn get_protocol(&self) -> &Protocol;
    fn get_last_check(&self) -> Option<std::time::Instant>;
    fn get_last_error(&self) -> Option<String>;
    
    fn set_health(&mut self, health: bool);
}
