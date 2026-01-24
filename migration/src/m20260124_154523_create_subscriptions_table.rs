use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Subscriptions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Subscriptions::Email).text().not_null().unique_key())
                    .col(ColumnDef::new(Subscriptions::Name).text().not_null())
                    .col(ColumnDef::new(Subscriptions::SubscribedAt).timestamp().not_null())
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
    Email,
    Name,
    SubscribedAt,
}
