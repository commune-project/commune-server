table! {
    actors (id) {
        id -> Int8,
        uri -> Varchar,
        url -> Nullable<Varchar>,
        kind -> Varchar,
        username -> Varchar,
        domain -> Varchar,
        name -> Varchar,
        summary -> Text,
        avatar_url -> Varchar,
        inbox_uri -> Varchar,
        outbox_uri -> Varchar,
        followers_uri -> Nullable<Varchar>,
        following_uri -> Nullable<Varchar>,
        public_key_pem -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        lang -> Varchar,
        is_locked -> Bool,
        is_suspended -> Bool,
        is_silenced -> Bool,
    }
}

table! {
    follows (follower_id, following_id) {
        follower_id -> Int8,
        following_id -> Int8,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        role -> Varchar,
    }
}

table! {
    users (actor_id) {
        actor_id -> Int8,
        email -> Nullable<Varchar>,
        is_email_verified -> Bool,
        password_hash -> Nullable<Varchar>,
        private_key_pem -> Text,
        register_ip -> Nullable<Varchar>,
        last_login_ip -> Nullable<Varchar>,
    }
}

joinable!(users -> actors (actor_id));

allow_tables_to_appear_in_same_query!(
    actors,
    follows,
    users,
);
