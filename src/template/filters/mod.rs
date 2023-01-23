mod insert_indent;

pub trait RingerFilter {
    const NAME: &'static str;
}

pub use insert_indent::InsertIndent;
