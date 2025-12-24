
pub mod setevent;
pub mod dyn_settings;


//Singleton that stores all major settings for the scouter (not dyn like event)
pub struct Settings {
    pub year: i32,
    pub bcrypt: u32,
    pub db_path: &'static str,
    pub blue_api_key: &'static str,
}