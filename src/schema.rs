// @generated automatically by Diesel CLI.

diesel::table! {
    indicator_scores (id) {
        id -> Int4,
        detail_id -> Int4,
        metric_key -> Varchar,
        score -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    indicator_sub_scores (id) {
        id -> Int4,
        indicator_score_id -> Int4,
        sub_score_type -> Varchar,
        score -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    metric_values (id) {
        id -> Int8,
        stock_id -> Int4,
        metric_key -> Varchar,
        period -> Varchar,
        period_end -> Date,
        value -> Float8,
        unit -> Nullable<Varchar>,
        currency -> Nullable<Varchar>,
        source -> Varchar,
        fetched_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    metrics_catalog (id) {
        id -> Int4,
        key -> Varchar,
        name -> Varchar,
        category -> Varchar,
        data_type -> Varchar,
        unit -> Nullable<Varchar>,
        frequency -> Nullable<Varchar>,
        higher_is_better -> Bool,
        min_plausible -> Nullable<Float8>,
        max_plausible -> Nullable<Float8>,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    portfolios (id) {
        id -> Int4,
        name -> Varchar,
        kind -> Varchar,
        #[max_length = 3]
        currency -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    score_details (id) {
        id -> Int4,
        snapshot_id -> Int4,
        category -> Varchar,
        score -> Float8,
        weight -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    score_snapshots (id) {
        id -> Int4,
        stock_id -> Int4,
        scored_at -> Timestamp,
        global_score -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sector_benchmarks (id) {
        id -> Int4,
        sector -> Varchar,
        industry -> Nullable<Varchar>,
        metric_key -> Varchar,
        value -> Float8,
        source -> Varchar,
        period_end -> Date,
        fetched_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    stocks (id) {
        id -> Int4,
        symbol -> Varchar,
        name -> Varchar,
        isin -> Varchar,
        currency -> Nullable<Varchar>,
        market -> Nullable<Varchar>,
        sector -> Nullable<Varchar>,
        industry -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(indicator_scores -> score_details (detail_id));
diesel::joinable!(indicator_sub_scores -> indicator_scores (indicator_score_id));
diesel::joinable!(metric_values -> stocks (stock_id));
diesel::joinable!(score_details -> score_snapshots (snapshot_id));
diesel::joinable!(score_snapshots -> stocks (stock_id));

diesel::allow_tables_to_appear_in_same_query!(
    indicator_scores,
    indicator_sub_scores,
    metric_values,
    metrics_catalog,
    portfolios,
    score_details,
    score_snapshots,
    sector_benchmarks,
    stocks,
);
