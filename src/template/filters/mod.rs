mod insert_indents;

pub trait RingerFilter {
    const NAME: &'static str;
}

pub use insert_indents::InsertIndents;
