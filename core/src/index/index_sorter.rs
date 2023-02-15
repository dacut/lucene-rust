use {
    crate::{
        index::{
            doc_values_iterator::DocValuesIterator, leaf_reader::LeafReader, numeric_doc_values::NumericDocValues,
            ordinal_map::OrdinalMapBase, sorted_doc_values::SortedDocValues,
        },
        search::doc_id_set_iterator::DocIdSetIterator,
        util::{
            long_values::LongValues,
            numeric_utils::{double_to_sortable_long, float_to_sortable_int},
            packed::packed_ints,
        },
    },
    pin_project::pin_project,
    std::{cmp::Ordering, future::Future, io::Result as IoResult, pin::Pin, sync::Arc},
};

/// Handles how documents should be sorted in an index, both within a segment and between segments.
///
/// Implementers must provide the following methods:
/// * [IndexSorter::get_doc_comparator]: an object that determines how documents within a segment are to be sorted
/// * [IndexSorter::get_comparable_providers]: an vec of objects that return a sortable long value per
///   document and segment
pub trait IndexSorter {
    /// The name of a [SortFieldProvider] that deserializes the parent [SortField]. This is used to maintain
    /// Java compatibility.
    fn get_provider_name(&self) -> &'static str;

    /// Get an vec of [ComparableProvider]s, one per segment, for merge sorting documents in
    /// different segments
    ///
    /// # Arguments
    /// `readers`: the readers to be merged
    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>>;

    /// Get a comparator that determines the sort order of docs within a single Reader.
    ///
    /// We cannot simply use the [FieldComparator] API because it requires docIDs to be
    /// sent in-order. The default implementations allocate array[maxDoc] to hold native values for
    /// comparison, but 1) they are transient (only alive while sorting this one segment) and 2) in the
    /// typical index sorting case, they are only used to sort newly flushed segments, which will be
    /// smaller than merged segments
    ///
    /// # Arguments
    /// `reader`: the Reader to sort
    /// `max_doc`: the number of documents in the Reader
    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>>;
}

/// Used for sorting documents across segments.
pub trait ComparableProvider {
    /// Returns an i64 so that the natural ordering of long values matches the ordering of doc IDs
    /// for the given comparator
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>>;
}

/// A comparator of doc IDs, used for sorting documents within a segment
pub trait DocComparator {
    /// A comparator of doc IDs, used for sorting documents within a segment
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering;
}

/// Provide a NumericDocValues instance for a LeafReader */
pub trait NumericDocValuesProvider {
    type NumericDocValues: NumericDocValues;

    /// Returns the NumericDocValues instance for this LeafReader
    fn get(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Self::NumericDocValues>>>>;
}

/// Provide a SortedDocValues instance for a LeafReader
pub trait SortedDocValuesProvider {
    /// Returns the SortedDocValues instance for this LeafReader
    fn get(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn SortedDocValues>>>>>>;
}

/// Sorts documents based on 32-bit integer values from a NumericDocValues instance
#[pin_project]
pub struct IntSorter<NDVP> {
    provider_name: &'static str,
    missing_value: Option<i32>,
    reverse: bool,
    #[pin]
    values_provider: NDVP,
}

impl<NDVP> IntSorter<NDVP> {
    /// Creates a new IntSorter.
    pub fn new(provider_name: &'static str, missing_value: Option<i32>, reverse: bool, values_provider: NDVP) -> Self {
        Self {
            provider_name,
            missing_value,
            reverse,
            values_provider,
        }
    }
}

impl<NDVP> IndexSorter for IntSorter<NDVP>
where
    NDVP: NumericDocValuesProvider,
{
    fn get_provider_name(&self) -> &'static str {
        self.provider_name
    }

    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let missing_value = this.missing_value.unwrap_or(0);

            let mut results = Vec::with_capacity(readers.len());
            for reader in readers {
                let values = this.values_provider.get(*reader).await?;
                let provider: Pin<Box<dyn ComparableProvider>> =
                    Box::pin(IntSorterComparableProvider::new(values, missing_value as i64));

                results.push(provider);
            }
            Ok(results)
        })
    }

    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let dvs = this.values_provider.get(reader).await?;
            let dvs = Box::pin(dvs);
            let values = Vec::with_capacity(max_doc);

            if let Some(mv) = this.missing_value {
                for _ in 0..max_doc {
                    values.push(*mv);
                }
            }

            loop {
                match dvs.as_mut().next_doc().await? {
                    None => break,
                    Some(doc_id) => values.push(dvs.as_ref().long_value().await? as i32),
                }
            }

            let provider: Pin<Box<dyn DocComparator>> = Box::pin(IntSorterDocComparator {
                values,
                reverse: *this.reverse,
            });

            Ok(provider)
        })
    }
}

#[pin_project]
pub(crate) struct IntSorterComparableProvider<NDV> {
    #[pin]
    values: NDV,
    missing_value: i64,
}

impl<NDV> IntSorterComparableProvider<NDV> {
    pub(crate) fn new(values: NDV, missing_value: i64) -> Self {
        Self {
            values,
            missing_value,
        }
    }
}

impl<NDV> ComparableProvider for IntSorterComparableProvider<NDV>
where
    NDV: NumericDocValues,
{
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        let this = self.project();
        Box::pin(async move {
            if this.values.as_mut().advance_exact(doc_id as i32).await? {
                this.values.as_ref().long_value().await
            } else {
                Ok(*this.missing_value)
            }
        })
    }
}

pub(crate) struct IntSorterDocComparator {
    values: Vec<i32>,
    reverse: bool,
}

impl DocComparator for IntSorterDocComparator {
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering {
        let v1 = self.values[doc_id1 as usize];
        let v2 = self.values[doc_id2 as usize];
        let result = v1.cmp(&v2);
        if self.reverse {
            result.reverse()
        } else {
            result
        }
    }
}

/// Sorts documents based on 64-bit integer values from a NumericDocValues instance
#[pin_project]
pub struct LongSorter<NDVP> {
    provider_name: &'static str,
    missing_value: Option<i64>,
    reverse: bool,
    #[pin]
    values_provider: NDVP,
}

impl<NDVP> LongSorter<NDVP> {
    /// Creates a new LongSorter.
    pub fn new(provider_name: &'static str, missing_value: Option<i64>, reverse: bool, values_provider: NDVP) -> Self {
        Self {
            provider_name,
            missing_value,
            reverse,
            values_provider,
        }
    }
}

impl<NDVP> IndexSorter for LongSorter<NDVP>
where
    NDVP: NumericDocValuesProvider,
{
    fn get_provider_name(&self) -> &'static str {
        self.provider_name
    }

    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let missing_value = this.missing_value.unwrap_or(0);
            let mut results = Vec::with_capacity(readers.len());
            for reader in readers {
                let values = this.values_provider.get(*reader).await?;
                let provider: Pin<Box<dyn ComparableProvider>> =
                    Box::pin(LongSorterComparableProvider::new(values, missing_value));
                results.push(provider);
            }
            Ok(results)
        })
    }

    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let dvs = this.values_provider.get(reader).await?;
            let dvs = Box::pin(dvs);
            let values = Vec::with_capacity(max_doc);

            if let Some(mv) = this.missing_value {
                for _ in 0..max_doc {
                    values.push(*mv);
                }
            }

            loop {
                match dvs.as_mut().next_doc().await? {
                    None => break,
                    Some(doc_id) => values.push(dvs.as_ref().long_value().await?),
                }
            }

            let provider: Pin<Box<dyn DocComparator>> = Box::pin(LongSorterDocComparator {
                values,
                reverse: *this.reverse,
            });

            Ok(provider)
        })
    }
}

#[pin_project]
struct LongSorterComparableProvider<NDV> {
    #[pin]
    values: NDV,
    missing_value: i64,
}

impl<NDV> LongSorterComparableProvider<NDV> {
    pub(crate) fn new(values: NDV, missing_value: i64) -> Self {
        Self {
            values,
            missing_value,
        }
    }
}

impl<NDV> ComparableProvider for LongSorterComparableProvider<NDV>
where
    NDV: NumericDocValues,
{
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        let this = self.project();
        Box::pin(async move {
            if this.values.as_mut().advance_exact(doc_id as i32).await? {
                this.values.as_ref().long_value().await
            } else {
                Ok(*this.missing_value)
            }
        })
    }
}

struct LongSorterDocComparator {
    values: Vec<i64>,
    reverse: bool,
}

impl DocComparator for LongSorterDocComparator {
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering {
        let v1 = self.values[doc_id1 as usize];
        let v2 = self.values[doc_id2 as usize];
        let result = v1.cmp(&v2);
        if self.reverse {
            result.reverse()
        } else {
            result
        }
    }
}

/// Sorts documents based on 32-bit floating-point values from a NumericDocValues instance
#[pin_project]
pub struct FloatSorter<NDVP> {
    provider_name: &'static str,
    missing_value: Option<f32>,
    reverse: bool,
    #[pin]
    values_provider: NDVP,
}

impl<NDVP> FloatSorter<NDVP> {
    /// Creates a new FloatSorter.
    pub fn new(provider_name: &'static str, missing_value: Option<f32>, reverse: bool, values_provider: NDVP) -> Self {
        Self {
            provider_name,
            missing_value,
            reverse,
            values_provider,
        }
    }
}

impl<NDVP> IndexSorter for FloatSorter<NDVP>
where
    NDVP: NumericDocValuesProvider,
{
    fn get_provider_name(&self) -> &'static str {
        self.provider_name
    }

    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let missing_value = this.missing_value.unwrap_or(0.0);
            let mut results = Vec::with_capacity(readers.len());
            for reader in readers {
                let values = this.values_provider.get(*reader).await?;
                let provider: Pin<Box<dyn ComparableProvider>> =
                    Box::pin(FloatSorterComparableProvider::new(values, missing_value));
                results.push(provider);
            }
            Ok(results)
        })
    }

    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let dvs = this.values_provider.get(reader).await?;
            let dvs = Box::pin(dvs);
            let values = Vec::with_capacity(max_doc);

            if let Some(mv) = this.missing_value {
                for _ in 0..max_doc {
                    values.push(*mv);
                }
            }

            loop {
                match dvs.as_mut().next_doc().await? {
                    None => break,
                    Some(doc_id) => values.push(f32::from_bits(dvs.as_ref().long_value().await? as u32)),
                }
            }

            let provider: Pin<Box<dyn DocComparator>> = Box::pin(FloatSorterDocComparator {
                values,
                reverse: *this.reverse,
            });

            Ok(provider)
        })
    }
}

#[pin_project]
struct FloatSorterComparableProvider<NDV> {
    #[pin]
    values: NDV,
    missing_value: f32,
}

impl<NDV> FloatSorterComparableProvider<NDV> {
    pub(crate) fn new(values: NDV, missing_value: f32) -> Self {
        Self {
            values,
            missing_value,
        }
    }
}

impl<NDV> ComparableProvider for FloatSorterComparableProvider<NDV>
where
    NDV: NumericDocValues,
{
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        let this = self.project();
        Box::pin(async move {
            let value = if this.values.as_mut().advance_exact(doc_id).await? {
                f32::from_bits(this.values.as_ref().long_value().await? as u32)
            } else {
                *this.missing_value
            };
            Ok(float_to_sortable_int(value) as i64)
        })
    }
}

struct FloatSorterDocComparator {
    values: Vec<f32>,
    reverse: bool,
}

impl DocComparator for FloatSorterDocComparator {
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering {
        let v1 = self.values[doc_id1 as usize];
        let v2 = self.values[doc_id2 as usize];
        match v1.partial_cmp(&v2) {
            Some(result) => {
                if self.reverse {
                    result.reverse()
                } else {
                    result
                }
            }

            None => {
                if v1.is_nan() {
                    if v2.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

/// Sorts documents based on 64-bit floating-point values from a NumericDocValues instance
#[pin_project]
pub struct DoubleSorter<NDVP> {
    provider_name: &'static str,
    missing_value: Option<f64>,
    reverse: bool,
    #[pin]
    values_provider: NDVP,
}

impl<NDVP> DoubleSorter<NDVP> {
    /// Creates a new DoubleSorter.
    pub fn new(provider_name: &'static str, missing_value: Option<f64>, reverse: bool, values_provider: NDVP) -> Self {
        Self {
            provider_name,
            missing_value,
            reverse,
            values_provider,
        }
    }
}

impl<NDVP> IndexSorter for DoubleSorter<NDVP>
where
    NDVP: NumericDocValuesProvider,
{
    fn get_provider_name(&self) -> &'static str {
        self.provider_name
    }

    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let missing_value = self.missing_value.unwrap_or(0.0);
            let mut results = Vec::with_capacity(readers.len());
            for reader in readers {
                let values = this.values_provider.get(*reader).await?;
                let provider: Pin<Box<dyn ComparableProvider>> =
                    Box::pin(DoubleSorterComparableProvider::new(values, missing_value));
                results.push(provider);
            }
            Ok(results)
        })
    }

    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let dvs = this.values_provider.get(reader).await?;
            let dvs = Box::pin(dvs);
            let values = Vec::with_capacity(max_doc);

            if let Some(mv) = this.missing_value {
                for _ in 0..max_doc {
                    values.push(*mv);
                }
            }

            loop {
                match dvs.as_mut().next_doc().await? {
                    None => break,
                    Some(doc_id) => values.push(f64::from_bits(dvs.as_ref().long_value().await? as u64)),
                }
            }

            let provider: Pin<Box<dyn DocComparator>> = Box::pin(DoubleSorterDocComparator {
                values,
                reverse: *this.reverse,
            });

            Ok(provider)
        })
    }
}

#[pin_project]
struct DoubleSorterComparableProvider<NDV> {
    #[pin]
    values: NDV,
    missing_value: f64,
}

impl<NDV> DoubleSorterComparableProvider<NDV> {
    pub(crate) fn new(values: NDV, missing_value: f64) -> Self {
        Self {
            values,
            missing_value,
        }
    }
}

impl<NDV> ComparableProvider for DoubleSorterComparableProvider<NDV>
where
    NDV: NumericDocValues,
{
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        let this = self.project();
        Box::pin(async move {
            let value = if this.values.as_mut().advance_exact(doc_id).await? {
                f64::from_bits(this.values.as_ref().long_value().await? as u64)
            } else {
                *this.missing_value
            };
            Ok(double_to_sortable_long(value) as i64)
        })
    }
}

struct DoubleSorterDocComparator {
    values: Vec<f64>,
    reverse: bool,
}

impl DocComparator for DoubleSorterDocComparator {
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering {
        let v1 = self.values[doc_id1 as usize];
        let v2 = self.values[doc_id2 as usize];
        match v1.partial_cmp(&v2) {
            Some(result) => {
                if self.reverse {
                    result.reverse()
                } else {
                    result
                }
            }

            None => {
                if v1.is_nan() {
                    if v2.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MissingOrder {
    First,
    Last,
}

impl Default for MissingOrder {
    fn default() -> Self {
        MissingOrder::First
    }
}

impl From<&MissingOrder> for i32 {
    fn from(missing_order: &MissingOrder) -> Self {
        match missing_order {
            MissingOrder::First => i32::MIN,
            MissingOrder::Last => i32::MAX,
        }
    }
}

/// Sorts documents based on terms from a SortedDocValues instance
#[pin_project]
pub struct StringSorter<SDVP> {
    provider_name: &'static str,
    missing_value: MissingOrder,
    reverse: bool,
    #[pin]
    values_provider: SDVP,
}

impl<SDVP> StringSorter<SDVP> {
    /// Creates a new StringSorter.
    pub fn new(provider_name: &'static str, missing_value: MissingOrder, reverse: bool, values_provider: SDVP) -> Self {
        Self {
            provider_name,
            missing_value,
            reverse,
            values_provider,
        }
    }
}

impl<SDVP> IndexSorter for StringSorter<SDVP>
where
    SDVP: SortedDocValuesProvider,
{
    fn get_provider_name(&self) -> &'static str {
        self.provider_name
    }

    fn get_comparable_providers(
        self: Pin<&Self>,
        readers: &[Arc<dyn LeafReader>],
    ) -> Pin<Box<dyn Future<Output = IoResult<Vec<Pin<Box<dyn ComparableProvider>>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let missing_value = this.missing_value;
            let mut values = Vec::with_capacity(readers.len());
            let mut results = Vec::with_capacity(readers.len());

            for reader in readers {
                let sorted = this.values_provider.get(*reader).await?;
                values.push(sorted);
            }

            let ordinal_map = OrdinalMapBase::from_value(&values, packed_ints::DEFAULT);
            let missing_ord: i32 = this.missing_value.into();

            for (reader_index, reader_values) in values.into_iter().enumerate() {
                let global_ords = ordinal_map.get_global_ords(reader_index);
                let provider: Pin<Box<dyn ComparableProvider>> = Box::pin(StringSorterComparableProvider {
                    reader_values,
                    global_ords,
                    missing_ord,
                });
                results.push(provider);
            }
            Ok(results)
        })
    }

    fn get_doc_comparator(
        self: Pin<&Self>,
        reader: Arc<dyn LeafReader>,
        max_doc: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn DocComparator>>>>>> {
        let this = self.project_ref();
        Box::pin(async move {
            let sorted = this.values_provider.get(reader).await?;
            let missing_ord = this.missing_value.into();
            let mut ords = vec![missing_ord; max_doc];

            while let Some(doc_id) = DocIdSetIterator::next_doc(sorted.as_mut()).await? {
                ords[doc_id as usize] = sorted.as_ref().ord_value().await?;
            }

            let provider: Pin<Box<dyn DocComparator>> = Box::pin(StringSorterDocComparator {
                ords,
                reverse: *this.reverse,
            });

            Ok(provider)
        })
    }
}

#[derive(Debug)]
struct StringSorterDocComparator {
    ords: Vec<i32>,
    reverse: bool,
}

impl DocComparator for StringSorterDocComparator {
    fn compare(&self, doc_id1: i32, doc_id2: i32) -> Ordering {
        let ord1 = self.ords[doc_id1 as usize];
        let ord2 = self.ords[doc_id2 as usize];
        let result = ord1.cmp(&ord2);

        if self.reverse {
            result.reverse()
        } else {
            result
        }
    }
}

pub struct StringSorterComparableProvider<'a> {
    reader_values: Pin<Box<dyn SortedDocValues + 'a>>,
    global_ords: Box<dyn LongValues + 'a>,
    missing_ord: i32,
}

impl<'a> ComparableProvider for StringSorterComparableProvider<'a> {
    fn get_as_comparable_long(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<i64>>>> {
        let this = self;
        Box::pin(async move {
            if DocValuesIterator::advance_exact(self.reader_values.as_mut(), doc_id).await? {
                // translate segment's ord to global ord space:
                Ok(this.global_ords.as_ref()[this.reader_values.as_ref().ord_value().await? as usize])
            } else {
                Ok(this.missing_ord as i64)
            }
        })
    }
}
