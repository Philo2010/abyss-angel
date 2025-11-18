use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/*
CREATE TABLE match_teams (
    id SERIAL PRIMARY KEY,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    team_number INTEGER NOT NULL,
    station TEXT NOT NULL
);
 */

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(DynSettings::Table)
                    .if_not_exists()
                    .col(pk_auto(DynSettings::Id))
                    .col(string(DynSettings::Event))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(DynSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DynSettings {
    Table,
    Id,
    Event,
}
