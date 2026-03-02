use database_macro::build_database;

use database::{
    AbsFieldConstraints, AbsFolderConstraints, AbsKeyConstraints, AccessorError, AllVariants,
    DataFieldAccessor, DataFieldPartialAccessor, DatabaseDescription, FlatFieldConstraints,
    FlatFolderConstraints, FlatKeyConstraints, FocusHandler, FolderAccessor, LayerDatabase, ToKey,
    UsizeConstraints, VariantCount,
};

build_database!(
    // ===== Base Classification =====
    struct Animalia {
        animalia_is_eukaryotic: usize,
        animalia_is_multicellular: u8,
        animalia_requires_oxygen: i8,
        animalia_can_move: usize,
        animalia_cell_complexity_index: usize,
    },

    // ===== Families =====
    struct Canidae {
        canidae_animalia: Animalia,
        canidae_pack_behavior_index: usize,
        canidae_average_lifespan_years: u64,
        canidae_diet_type_index: usize,
        canidae_social_structure_rating: usize,
    },

    struct Felidae {
        felidae_animalia: Animalia,
        felidae_claw_sharpness_index: usize,
        felidae_night_vision_rating: usize,
        felidae_agility_score: usize,
        felidae_average_weight_kg: u8,
    },

    struct Hominidae {
        hominidae_animalia: Animalia,
        hominidae_brain_volume_index: usize,
        hominidae_tool_usage_rating: usize,
        hominidae_social_complexity_index: usize,
        hominidae_bipedal_efficiency_score: usize,
    },

    struct Corvidae {
        corvidae_animalia: Animalia,
        corvidae_problem_solving_index: u8,
        corvidae_memory_capacity_score: usize,
        corvidae_wingspan_cm: u32,
        corvidae_sound_mimic_rating: usize,
    },

    // ===== Animals =====
    struct Dog {
        dog_canidae: Canidae,
        dog_domestication_level: usize,
        dog_average_weight_kg: usize,
        dog_bark_volume_index: i16,
        dog_tail_length_cm: usize,
        dog_energy_level_score: usize,
    },

    struct Wolf {
        wolf_canidae: Canidae,
        wolf_pack_rank_index: usize,
        wolf_average_weight_kg: usize,
        wolf_hunting_success: bool,
        wolf_territory_size_km2: usize,
        wolf_howl_frequency_index: usize,
    },

    struct Cat {
        cat_felidae: Felidae,
        cat_climbing_skill_index: usize,
        cat_sleep_hours_daily: u8,
        cat_jump_height_cm: usize,
        cat_purr_frequency_index: usize,
        cat_independence_rating: usize,
    },

    struct Human {
        human_hominidae: Hominidae,
        human_average_height_cm: u16,
        human_language_count: usize,
        human_technology_level_index: usize,
        human_lifespan_expectancy_years: u8,
        human_problem_solving_score: usize,
    },

    struct Crow {
        crow_corvidae: Corvidae,
        crow_tool_usage_index: usize,
        crow_flight_altitude_m: u16,
        crow_call_variation_count: usize,
        crow_learning_speed_score: u8,
        crow_social_behavior_index: usize,
    },

    // ===== Root =====
    struct Root {
        root_dog: Dog,
        root_wolf: Wolf,
        root_cat: Cat,
        root_human: Human,
        root_crow: Crow,
    }
);

fn main() {}
