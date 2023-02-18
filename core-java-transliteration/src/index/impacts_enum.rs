use {
    crate::index::{postings_enum::PostingsEnum, impacts_source::ImpactsSource},
};

pub trait ImpactsEnum: PostingsEnum + ImpactsSource {}
