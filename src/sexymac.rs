use sea_orm::{DatabaseConnection, EntityTrait};


#[macro_export]
macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin($body) as Pin<Box<dyn Future<Output = _> + Send>>
    };
}


pub async fn get_event_default(db: &DatabaseConnection) -> Option<String> {
    match crate::entity::dyn_settings::Entity::find().one(db).await {
        Ok(a) => {
            a.map(|x| x.event)
        },
        Err(a) => {
            println!("Failed to get event! {a}");
            None
        },
    }
}