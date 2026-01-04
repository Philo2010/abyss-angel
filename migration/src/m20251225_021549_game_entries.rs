//This is the genertic header for all years of FRC, this will then link to a per year table contanting per year data


//TODO: Write code for entiry then handle the merging of different years

//Enum for levels of tournaments 
#[derive(DeriveIden)]
enum TournamentLevels {
    Table,
    QualificationMatch, //Blue api defines this as "qm"
    Quarterfinal,       //Blue api defines this as "qf"
    Semifinal,          //Blue api defines this as "sf"
    Final,              //Blue api defines this as "f"
}

//Enum for Stations
#[derive(DeriveIden)]
enum Stations {
    Table,
    Red1,
    Red2,
    Red3,
    Blue1,
    Blue2,
    Blue3
}


use sea_orm_migration::{prelude::{extension::postgres::Type, *}, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {


        //For levels of tournaments 
        manager
            .create_type(
                Type::create()
                    .as_enum(TournamentLevels::Table)
                    .values([
                        TournamentLevels::QualificationMatch,
                        TournamentLevels::Quarterfinal,
                        TournamentLevels::Semifinal,
                        TournamentLevels::Final
                    ])
                    .to_owned()
            ).await?;

        //For different Stations
        manager
            .create_type(
                Type::create()
                    .as_enum(Stations::Table)
                    .values([
                        Stations::Red1,
                        Stations::Red2,
                        Stations::Red3,
                        Stations::Blue1,
                        Stations::Blue2,
                        Stations::Blue3,
                    ])
                    .to_owned()
            ).await?;

        manager
            .create_table(
                Table::create()
                    .table(GenerticHeader::Table)
                    .if_not_exists()
                    .col(pk_auto(GenerticHeader::Id).not_null())
                    .col(uuid(GenerticHeader::User).not_null())
                    .col(integer(GenerticHeader::Team).not_null())
                    .col(boolean(GenerticHeader::IsABTeam).not_null())
                    .col(integer(GenerticHeader::MatchId).not_null())
                    .col(integer(GenerticHeader::Set).not_null())
                    .col(integer(GenerticHeader::TotalScore).not_null())
                    .col(string(GenerticHeader::EventCode).not_null())
                    .col(custom(GenerticHeader::TournamentLevel, TournamentLevels::Table).not_null())
                    .col(custom(GenerticHeader::Station, Stations::Table).not_null())
                    .col(date_time(GenerticHeader::CreatedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        manager
            .drop_type(Type::drop()
                .name(TournamentLevels::Table)
                .to_owned()
                )
            .await?;

        manager
            .drop_type(Type::drop()
                .name(Stations::Table)
                .to_owned()
                )
            .await?;

        manager
            .drop_table(Table::drop().table(GenerticHeader::Table).to_owned())
            .await
    }
}



#[derive(DeriveIden)]
enum GenerticHeader {
    Table,
    Id, //
    User, //
    Team, //
    IsABTeam, //
    MatchId, //
    Set, //
    TotalScore, //
    EventCode, //
    TournamentLevel, //
    Station,  //
    CreatedAt,
}
