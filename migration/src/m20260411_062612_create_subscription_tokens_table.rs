use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionTokens::Table)
                    .if_not_exists()
                    // 👉 自增主键（核心）
                    .col(
                        ColumnDef::new(SubscriptionTokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // 👉 token（业务唯一）
                    .col(
                        ColumnDef::new(SubscriptionTokens::SubscriptionToken)
                            .string_len(255)
                            .not_null()
                            .default("")
                            .unique_key(),
                    )
                    // 👉 外键字段
                    .col(
                        ColumnDef::new(SubscriptionTokens::SubscriberId)
                            .integer()
                            .not_null(),
                    )
                    // 👉 外键约束
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-token-subscriber")
                            .from(SubscriptionTokens::Table, SubscriptionTokens::SubscriberId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SubscriptionTokens::Table)
                    .if_exists() // 👉 防炸关键
                    .to_owned(),
            )
            .await
    }
}

// 👉 表定义
#[derive(DeriveIden)]
enum SubscriptionTokens {
    Table,
    Id,
    SubscriptionToken,
    SubscriberId,
}

// 👉 引用 subscriptions 表
#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
}
