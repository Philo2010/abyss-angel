

#[macro_export]
macro_rules! define_pits {
    ($($name:ident => $($module:ident)::+),* $(,)?) => {
        use rocket::serde::{Serialize, Deserialize};
        use schemars::JsonSchema;
        #[derive(Serialize, Deserialize, Clone, JsonSchema)]
        pub enum PitSpecific {
            $(
                $name($($module)::+::Model),
            )*
        }

        #[derive(Serialize, Deserialize, Clone, JsonSchema)]
        pub enum PitInsertsSpecific {
            $(
                $name($($module)::+::Insert),
            )*
        }
        
        #[derive(Serialize, Deserialize, Clone, JsonSchema)]
        pub enum PitEditSpecific {
            $(
                $name($($module)::+::Edit),
            )*
        }

        fn pit_dispatch(year_id: i32) -> Box<dyn PitScoutStandard> {
            match year_id {
                $(
                    $($module)::+::YEAR => {
                        Box::new($($module)::+::Functions) as Box<dyn PitScoutStandard>
                    },
                )*
                _ => panic!("Unknown year_id: {}", year_id),
            }
        }
    };
}