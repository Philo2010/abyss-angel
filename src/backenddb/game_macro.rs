#[macro_export]
macro_rules! define_games {
    ($($name:ident => $($module:ident)::+),* $(,)?) => {
        use serde::Deserialize;
        #[derive(Serialize, JsonSchema)]
        pub enum GamesFullSpecific {
            $(
                $name($($module)::+::Model),
            )*
        }
        
        #[derive(Serialize, JsonSchema)]
        pub enum GamesAvgSpecific {
            $(
                $name($($module)::+::Avg),
            )*
        }
        
        #[derive(Serialize, JsonSchema)]
        pub enum GamesGraphSpecific {
            $(
                $name($($module)::+::Graph),
            )*
        }
        
        #[derive(Serialize, Deserialize, JsonSchema)]
        pub enum GamesInsertsSpecific {
            $(
                $name($($module)::+::Insert),
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        pub enum GamesEditSpecific {
            $(
                $name($($module)::+::Edit),
            )*
        }

        fn game_dispatch(year_id: i32) -> Box<dyn YearOp> {
            match year_id {
                $(
                    $($module)::+::YEAR => {
                        Box::new($($module)::+::Functions) as Box<dyn YearOp>
                    },
                )*
                _ => panic!("Unknown year_id: {}", year_id),
            }
        }
    };
}