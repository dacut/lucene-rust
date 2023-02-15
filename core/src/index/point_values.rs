use {
    crate::{index::index_reader::IndexReader, util::bkd::bkd_config},
    std::{error::Error, io::Result as IoResult, sync::Arc},
};

/// Used by [PointValues::intersect] to check how each recursive cell corresponds to the query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    /// The cell is fully contained by the query
    CellInsideQuery,

    /// The cell and query do not overlap
    CellOutsideQuery,

    /// The cell partially overlaps the query
    CellCrossesQuery,
}

pub const MAX_NUM_BYTES: usize = 16;

pub const MAX_DIMENSIONS: usize = bkd_config::MAX_DIMS;

pub const MAX_INDEX_DIMENSIONS: usize = bkd_config::MAX_INDEX_DIMS;

/// Access to indexed numeric values.
///
/// Points represent numeric values and are indexed differently than ordinary text. Instead of an
/// inverted index, points are indexed with datastructures such as [KD-trees](https://en.wikipedia.org/wiki/K-d_tree).
/// These structures are optimized for operations such as _range_, _distance_, _nearest-neighbor_, and
/// _point-in-polygon_ queries.
///
/// # Basic Point Types
///
/// <table>
///   <caption>Basic point types in Java and Lucene</caption>
///   <tr><th>Java type</th><th>Lucene class</th></tr>
///   <tr><td>{@code int}</td><td>{@link IntPoint}</td></tr>
///   <tr><td>{@code long}</td><td>{@link LongPoint}</td></tr>
///   <tr><td>{@code float}</td><td>{@link FloatPoint}</td></tr>
///   <tr><td>{@code double}</td><td>{@link DoublePoint}</td></tr>
///  <tr><td>{@code byte[]}</td><td>{@link BinaryPoint}</td></tr>
///    <tr><td>{@link InetAddress}</td><td>{@link InetAddressPoint}</td></tr>
///   <tr><td>{@link BigInteger}</td><td><a href="{@docRoot}/../sandbox/org/apache/lucene/sandbox/document/BigIntegerPoint.html">BigIntegerPoint</a>*</td></tr>
/// </table>
///
//// * in the <i>lucene-sandbox</i> jar<br>
///
/// Basic Lucene point types behave like their java peers: for example {@link IntPoint} represents
/// a signed 32-bit {@link Integer}, supporting values ranging from {@link Integer#MIN_VALUE} to
/// {@link Integer#MAX_VALUE}, ordered consistent with {@link Integer#compareTo(Integer)}. In
/// addition to indexing support, point classes also contain static methods (such as {@link
/// IntPoint#newRangeQuery(String, int, int)}) for creating common queries. For example:
///
///
///   // add year 1970 to document
///    document.add(new IntPoint("year", 1970));
///    // index document
///   writer.addDocument(document);
///   ...
///    // issue range query of 1960-1980
///   Query query = IntPoint.newRangeQuery("year", 1960, 1980);
///   TopDocs docs = searcher.search(query, ...);
///
///
/// # Geospatial Point Types
///
/// Although basic point types such as {@link DoublePoint} support points in multi-dimensional space
/// too, Lucene has specialized classes for location data. These classes are optimized for location
/// data: they are more space-efficient and support special operations such as _distance_ and
/// _polygon_ queries. There are currently two implementations:
///
/// * [LatLonPoint]: indexes `(latitude,longitude)` as `(x,y)` two-dimensional space.
/// * `Geo3DPoint` in [lucene-spatial3d]: indexes `(latitude,longitude)` as `(x,y,z)` in three-dimensional space.
///    Does **not** support altitude, 3D here means "uses three dimensions under-the-hood"<br>
///
/// #Advanced usage<
///
/// Custom structures can be created on top of single- or multi- dimensional basic types, on top of
/// [BinaryPoint] for more flexibility, or via custom [Field] trait implementations.
///
/// @lucene.experimental
pub trait PointValues {
    /// Returns minimum value for each dimension, packed, or `None` if [::size] is 0.
    fn get_min_packed_value(&self) -> IoResult<Option<Vec<u8>>>;

    /// Returns maximum value for each dimension, packed, or `None` if [::size] is 0.
    fn get_max_packed_value(&self) -> IoResult<Option<Vec<u8>>>;

    /// Returns how many dimensions are represented in the values
    fn get_num_dimensions(&self) -> IoResult<usize>;

    /// Returns how many dimensions are used for the index
    fn get_num_index_dimensions(&self) -> IoResult<usize>;

    /// Returns the number of bytes per dimension
    fn get_bytes_per_dimension(&self) -> IoResult<usize>;

    /// Returns the total number of indexed points across all documents.
    fn size(&self) -> usize;

    /// Returns the total number of documents that have indexed at least one point.
    fn get_doc_count(&self) -> usize;
}

/// Return the cumulated number of points across all leaves of the given [IndexReader].
/// Leaves that do not have points for the given field are ignored.
///
/// # See
/// [PointValues::size]
fn size(reader: Arc<dyn IndexReader>, field: &str) -> IoResult<usize> {
    let mut size = 0;

    for ctx in reader.leaves()? {
        let reader = ctx.get_reader();
        let values = reader.as_ref().get_point_values(field)?;
        if let Some(values) = values {
            size += values.size();
        }
    }

    Ok(size)
}

/// Return the cumulated number of docs that have points across all leaves of the given [IndexReader].
/// Leaves that do not have points for the given field are ignored.
fn get_doc_count(reader: Arc<dyn IndexReader>, field: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let mut count = 0;
    for ctx in reader.leaves()? {
        let values = ctx.get_reader().get_point_values(field)?;
        if let Some(values) = values {
            count += values.get_doc_count();
        }
    }

    Ok(count)
}
