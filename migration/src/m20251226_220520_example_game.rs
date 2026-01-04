use sea_orm_migration::{prelude::*, schema::*};


//An example game with only 2 types hehe: int, and hoohoo: string
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(ExampleGame::Table)
                    .if_not_exists()
                    .col(pk_auto(ExampleGame::Id))
                    .col(integer(ExampleGame::Hehe))
                    .col(string(ExampleGame::Hoohoo))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(ExampleGame::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExampleGame {
    Table,
    Id,
    Hehe,
    Hoohoo,
}
