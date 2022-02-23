use crate::model;

pub trait Repository {
    fn add(&self, batch: model::Batch);

    fn get(&self, reference: i32) -> model::Batch;
}
