use commune::apub::models::PagedCollection;
use serde_urlencoded::{from_str, de::Error};

#[test]
fn test_deserialize_paged_collection_query_string() -> Result<(), Error> {
    let paged_collection: PagedCollection = from_str("")?;
    assert_eq!(paged_collection.page_number(), 0);
    let paged_collection: PagedCollection = from_str("page=1")?;
    assert_eq!(paged_collection.page_number(), 1);
    Ok(())
}