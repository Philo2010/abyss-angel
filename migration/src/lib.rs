pub use sea_orm_migration::prelude::*;

mod m20220101_000001_settings;
mod m20251118_075143_upcoming_games;
mod m20251130_100818_admins;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_settings::Migration),
            Box::new(m20251118_075143_upcoming_games::Migration),
            Box::new(m20251130_100818_admins::Migration),
        ]
    }
}
