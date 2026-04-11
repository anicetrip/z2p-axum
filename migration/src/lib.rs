pub use sea_orm_migration::prelude::*;

mod m20260124_154523_create_subscriptions_table;
mod m20260408_131915_add_status_to_subscriptions;
mod m20260411_062612_create_subscription_tokens_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260124_154523_create_subscriptions_table::Migration),
            Box::new(m20260408_131915_add_status_to_subscriptions::Migration),
            Box::new(m20260411_062612_create_subscription_tokens_table::Migration),
        ]
    }
}
