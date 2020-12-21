table! {
    climbs (id) {
        id -> Integer,
        hill -> Integer,
        warrior -> Integer,
        status -> Integer,
    }
}

table! {
    hill_warriors (id) {
        id -> Integer,
        hill -> Integer,
        warrior -> Integer,
        rank -> Integer,
        win -> Float,
        loss -> Float,
        tie -> Float,
        score -> Float,
    }
}

table! {
    hills (id) {
        id -> Integer,
        name -> Text,
        key -> Text,
        instruction_set -> Integer,
        core_size -> Integer,
        max_cycles -> Integer,
        max_processes -> Integer,
        max_warrior_length -> Integer,
        min_distance -> Integer,
        rounds -> Integer,
        slots -> Integer,
    }
}

table! {
    warriors (id) {
        id -> Integer,
        name -> Text,
        author -> Text,
        redcode -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    climbs,
    hill_warriors,
    hills,
    warriors,
);
