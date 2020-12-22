table! {
    authors (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    battles (id) {
        id -> Integer,
        hash -> Text,
        hill -> Integer,
        warrior_a -> Integer,
        warrior_b -> Integer,
        a_win -> Integer,
        a_loss -> Integer,
        a_tie -> Integer,
        b_win -> Integer,
        b_loss -> Integer,
        b_tie -> Integer,
    }
}

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
        hill -> Integer,
        author -> Integer,
        redcode -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    authors,
    battles,
    climbs,
    hill_warriors,
    hills,
    warriors,
);
