// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Integer,
        chat -> BigInt,
        selection -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> BigInt,
        chat -> Nullable<BigInt>,
        admin -> Bool,
        language -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(posts, users,);
