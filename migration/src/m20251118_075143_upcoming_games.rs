use sea_orm_migration::{prelude::*, schema::*};

/*
CREATE TABLE matches (
    id SERIAL PRIMARY KEY,
    event_code TEXT NOT NULL,
    match_number INTEGER NOT NULL,
    description TEXT NOT NULL,
    tournament_level TEXT NOT NULL
);

CREATE TABLE match_teams (
    id SERIAL PRIMARY KEY,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    team_number INTEGER NOT NULL,
    station TEXT NOT NULL
); */

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        match manager
            .create_table(
                Table::create()
                    .table(UpcomingGame::Table)
                    .if_not_exists()
                    .col(pk_auto(UpcomingGame::Id))
                    .col(string(UpcomingGame::EventCode))
                    .col(integer(UpcomingGame::MatchNumber))
                    .col(integer(UpcomingGame::SetNumber))
                    .col(string(UpcomingGame::TournamentLevel))
                    .to_owned(),
            )
            .await {
                Ok(_) => {},
                Err(a) => {return Err(a);},
            }

        manager
            .create_table(
                Table::create()
                    .table(UpcomingTeam::Table)
                    .if_not_exists()
                    .col(pk_auto(UpcomingTeam::Id))
                    .col(string(UpcomingTeam::Station))
                    .col(integer(UpcomingTeam::Team))
                    .col(string(UpcomingTeam::Scouter).null())
                    .col(integer(UpcomingTeam::GameId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_upcomingteam_game")
                            .from(UpcomingTeam::Table, UpcomingTeam::GameId)
                            .to(UpcomingGame::Table, UpcomingGame::Id)
                            .on_delete(ForeignKeyAction::Cascade), // <-- DELETE CASCADE here
                    )
                    .to_owned()
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(UpcomingGame::Table).to_owned())
            .await;
        
        manager
            .drop_table(Table::drop().table(UpcomingTeam::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UpcomingGame {
    Table,
    Id,
    EventCode,
    MatchNumber,
    SetNumber,
    Description,
    TournamentLevel,
}

#[derive(DeriveIden)]
enum  UpcomingTeam {
    Table,
    Id,
    Station,
    Team,
    Scouter,
    GameId
}