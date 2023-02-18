use {
    crate::{
        index::{
            index_reader::CacheKey, sorted_doc_values::SortedDocValues, sorted_set_doc_values::SortedSetDocValues,
            terms_enum::TermsEnum,
        },
        util::{
            long_values::{Identity, LongValues, Zeroes},
            packed::{
                monotonic_long_values::{Builder as MonotonicLongValuesBuilder, MonotonicLongValues},
                packed_ints::{bits_required, get_mutable, Mutable, COMPACT},
                packed_long_values::{Builder as PackedLongValuesBuilder, PackedLongValues, DEFAULT_PAGE_SIZE},
            },
        },
    },
    std::{
        cmp::{Ord, Ordering, PartialOrd},
        collections::{BinaryHeap, binary_heap::PeekMut},
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        future::Future,
        io::Result as IoResult,
        ops::Index,
        pin::Pin,
    },
};

pub use crate::util::packed::packed_ints::DEFAULT as DEFAULT_ACCEPTABLE_OVERHEAD_RATIO;

/// Maps per-segment ordinals to/from global ordinal space, using a compact packed-ints
/// representation.
///
/// # Note
/// This is a costly operation, as it must merge sort all terms, and may require
/// non-trivial RAM once done. It's better to operate in segment-private ordinal space instead when
/// possible.
pub trait OrdinalMap<'a> {
    /// Given a segment number, return a [LongValues] instance that maps segment ordinals to
    /// global ordinals.
    fn get_global_ords(&self, segment_index: i32) -> Box<dyn LongValues>;

    /// Given global ordinal, returns the ordinal of the first segment which contains this ordinal (the
    /// corresponding to the segment return [get_first_segment_number]
    fn get_first_segment_ord(&self, global_ord: i64) -> i64;

    /// Given a global ordinal, returns the index of the first segment that contains this term.
    fn get_first_segment_number(&self, global_ord: i64) -> i32;

    /// Returns the total number of unique terms in global ord space.
    fn get_value_count(&self) -> usize;
}

#[derive(Debug)]
pub struct TermsEnumIndex<'a> {
    pub(super) sub_index: u32,
    pub(super) terms_enum: &'a Pin<Box<dyn TermsEnum + 'a>>,
    pub(super) current_term: Vec<u8>,
}

impl<'a> TermsEnumIndex<'a> {
    pub fn new(terms_enum: &'a Pin<Box<dyn TermsEnum +'a>>, sub_index: u32) -> Self {
        Self {
            sub_index,
            terms_enum,
            current_term: Vec::new(),
        }
    }
}

impl<'a> TermsEnumIndex<'a> {
    pub fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self;
        Box::pin(async move { this.terms_enum.as_mut().next().await })
    }
}

#[derive(Debug)]
pub struct SegmentMap {
    /// Index from into weights, sorted by weight. That is, for each _i_:
    /// `weights[new_to_old[i]] <= weights[new_to_old[i + 1]]`.
    /// 
    /// If `weights = [50, 10, 20, 60, 30]`, then `new_to_old = [1, 2, 4, 0, 3]`.
    new_to_old: Vec<u32>,

    /// Indicates the relative order of weights; a reverse mapping of `new_to_old`.
    /// That is, for each _i_: if `old_to_new[i] <= old_to_new[j]` then `weights[i] <= weights[j]`.
    ///
    /// If `weights = [50, 10, 20, 60, 30]` and `new_to_old = [1, 2, 4, 0, 3]`, then
    /// `old_to_new = [3, 0, 1, 4, 2]`.
    old_to_new: Vec<u32>,
}

impl SegmentMap {
    /// Build a map from an index into a sorted view of `weights` to an index into `weights`.
    fn map(weights: &[i64]) -> Vec<u32> {
        let mut new_to_old = Vec::with_capacity(weights.len());
        for i in 0..weights.len() {
            new_to_old.push(i as u32);
        }

        new_to_old.sort_by_key(|&i| weights[i as usize]);
        new_to_old
    }

    /// Inverse the map
    fn inverse(map: &[u32]) -> Vec<u32> {
        let mut inverse = vec![0; map.len()];
        for i in 0..map.len() {
            inverse[map[i] as usize] = i as u32;
        }
        inverse
    }

    pub(super) fn new(weights: &[i64]) -> Self {
        let new_to_old = Self::map(weights);
        let old_to_new = Self::inverse(&new_to_old);
        Self {
            new_to_old,
            old_to_new,
        }
    }

    pub(super) fn get_new_to_old(&self, segment: u32) -> u32 {
        self.new_to_old[segment as usize]
    }

    pub(super) fn get_old_to_new(&self, segment: u32) -> u32 {
        self.old_to_new[segment as usize]
    }
}

#[derive(Debug)]
pub struct OrdinalMapBase {
    /// Cache key of whoever asked for this awful thing
    owner: CacheKey,

    /// Number of global ordinals
    value_count: usize,

    /// globalOrd -> (globalOrd - segmentOrd) where segmentOrd is the ordinal in the first segment
    /// that contains this term
    global_ord_deltas: Box<dyn LongValues>,

    /// globalOrd -> first segment container
    first_segments: Box<dyn LongValues>,

    /// for every segment, segmentOrd -> globalOrd
    segment_to_global_ords: Vec<Box<dyn LongValues>>,

    /// the map from/to segment ids
    segment_map: SegmentMap,
}

impl OrdinalMapBase {
    /// Build a new [OrdinalMapBase]. This is the package-private constructor in Java with the signature:
    /// `OrdinalMapBase(IndexReader.CacheKey, TermsEnum[], SegmentMap, float)`.
    pub(crate) async fn from_segment_map<'a>(
        owner: CacheKey,
        subs: &'a [Pin<Box<dyn TermsEnum + 'a>>],
        segment_map: SegmentMap,
        acceptable_overhead_ratio: f32,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        // create the ordinal mappings by pulling a termsenum over each sub's
        // unique terms, and walking a multitermsenum over those

        // this.owner = owner;
        // this.segmentMap = segmentMap;

        // even though we accept an overhead ratio, we keep these ones with COMPACT
        // since they are only used to resolve values given a global ord, which is
        // slow anyway
        let mut global_ord_deltas = MonotonicLongValuesBuilder::new(DEFAULT_PAGE_SIZE, COMPACT)?;
        let mut first_segments = PackedLongValuesBuilder::new(DEFAULT_PAGE_SIZE, COMPACT)?;
        let mut first_segment_bits = 0;
        let mut ord_deltas = Vec::with_capacity(subs.len());

        for _ in 0..subs.len() {
            ord_deltas.push(MonotonicLongValuesBuilder::new(DEFAULT_PAGE_SIZE, acceptable_overhead_ratio)?);
        }

        let mut ord_delta_bits = Vec::with_capacity(subs.len());
        let mut segment_ords = Vec::<u64>::with_capacity(subs.len());

        // Just merge-sorts by term:
        struct TermsEnumIndexSorter<'a>(Pin<Box<TermsEnumIndex<'a>>>);
        impl<'a> PartialEq for TermsEnumIndexSorter<'a> {
            fn eq(&self, other: &Self) -> bool {
                self.0.as_ref().current_term == other.0.as_ref().current_term
            }
        }
        impl<'a> Eq for TermsEnumIndexSorter<'a> {}
        impl<'a> PartialOrd for TermsEnumIndexSorter<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.0.as_ref().current_term.cmp(&other.0.as_ref().current_term))
            }
        }
        impl<'a> Ord for TermsEnumIndexSorter<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.as_ref().current_term.cmp(&other.0.as_ref().current_term)
            }
        }

        let mut queue = BinaryHeap::with_capacity(subs.len());

        for i in 0..subs.len() {
            let sub = &subs[segment_map.get_new_to_old(i as u32) as usize];
            let mut sub = Box::pin(TermsEnumIndex::new(sub, i as u32));
            if sub.as_mut().next().await?.is_some() {
                queue.push(TermsEnumIndexSorter(sub));
            }
        }

        let mut scratch = Vec::new();
        let mut global_ord = 0;

        while let Some(top) = queue.pop() {
            let top = top.0;
            scratch.clear();
            scratch.extend_from_slice(&top.current_term);

            let mut first_segment_index = u32::MAX;
            let mut global_ord_delta = u64::MAX;

            // Advance past this term, recording the per-segment ord deltas:
            loop {
                let mut top = queue.peek_mut().unwrap();
                let segment_ord = top.0.terms_enum.as_ref().ord().await?;
                let delta = global_ord - segment_ord;
                let segment_index = top.0.sub_index;

                // We compute the least segment where the term occurs. In case the
                // first segment contains most (or better all) values, this will
                // help save significant memory
                if segment_index < first_segment_index {
                    first_segment_index = segment_index;
                    global_ord_delta = delta;
                }

                ord_delta_bits.as_mut_slice()[segment_index as usize] |= delta;

                // for each per-segment ord, map it back to the global term; the while loop is needed
                // in case the incoming TermsEnums don't have compact ordinals (some ordinal values
                // are skipped), which can happen e.g. with a FilteredTermsEnum:
                assert!(segment_ords.as_slice()[segment_index as usize] <= segment_ord);

                // TODO: we could specialize this case (the while loop is not needed when the ords
                // are compact)
                loop {
                    ord_deltas.as_mut_slice()[segment_index as usize].add(delta as i64);
                    segment_ords.as_mut_slice()[segment_index as usize] += 1;

                    if segment_ords.as_slice()[segment_index as usize] > segment_ord {
                        break;
                    }
                }

                let has_more = top.0.as_mut().next().await?.is_some();

                if !has_more {
                    PeekMut::pop(top);
                    
                    if queue.is_empty() {
                        break;
                    }
                } else {
                    drop(top);
                }

                if queue.peek().unwrap().0.current_term != scratch {
                    break;
                }
            }

            // for each unique term, just mark the first segment index/delta where it occurs
            first_segments.add(first_segment_index as i64);
            first_segment_bits |= first_segment_index;
            global_ord_deltas.add(global_ord_delta as i64);
            global_ord += 1;
        }

        let value_count = global_ord as usize;

        // If the first segment contains all of the global ords, then we can apply a small optimization
        // and hardcode the first segment indices and global ord deltas as all zeroes.
        let (first_segments, global_ord_deltas): (Box<dyn LongValues>, Box<dyn LongValues>) =
            if ord_delta_bits.first() == Some(&0) && first_segment_bits == 0 {
                (Box::new(Zeroes), Box::new(Zeroes))
            } else {
                let packed_first_segments = first_segments.build();
                let packed_global_ord_deltas = global_ord_deltas.build();
                (Box::new(packed_first_segments), Box::new(packed_global_ord_deltas))
            };

        // ord_deltas is typically the bottleneck, so let's see what we can do to make it faster
        let mut segment_to_global_ords: Vec<Box<dyn LongValues>> = Vec::with_capacity(subs.len());

        for (i, ord_delta) in ord_deltas.into_iter().enumerate() {
            let deltas = ord_delta.build();
            if ord_delta_bits.as_slice()[i] == 0 {
                // segment ords perfectly match global ordinals
                // likely in case of low cardinalities and large segments
                segment_to_global_ords.as_mut_slice()[i] = Box::new(Identity);
            } else {
                let bits_required = if ord_delta_bits.as_slice()[i] < 0 {
                    64
                } else {
                    bits_required(ord_delta_bits.as_slice()[i] as i64)
                };

                // FIXME: Do we need to implement ram_bytes_used everywhere?
                // let monotonic_bits = deltas.ram_bytes_used() * 8;
                let packed_bits = bits_required as usize * deltas.size();

                if deltas.size() <= i32::MAX as usize
                /* && packed_bits <= monotonic_bits * (1.0 + acceptable_overhead_ratio */
                {
                    // monotonic compression mostly adds overhead, let's keep the mapping in plain packed ints
                    let size = deltas.size();
                    let mut new_deltas = get_mutable(size as u32, bits_required, acceptable_overhead_ratio);
                    for (ord, el) in deltas.iter().enumerate() {
                        new_deltas.set(ord, el);
                    }

                    #[derive(Debug)]
                    struct MutableLongValuesWrapper(Box<dyn Mutable>);
                    impl Index<usize> for MutableLongValuesWrapper {
                        type Output = i64;

                        fn index(&self, index: usize) -> &Self::Output {
                            &self.0.get(index)
                        }
                    }
                    impl LongValues for MutableLongValuesWrapper {}

                    segment_to_global_ords.as_mut_slice()[i] = Box::new(MutableLongValuesWrapper(new_deltas));
                } else {
                    #[derive(Debug)]
                    struct DeltaLongValuesWrapper(MonotonicLongValues);
                    impl Index<usize> for DeltaLongValuesWrapper {
                        type Output = i64;

                        fn index(&self, index: usize) -> &Self::Output {
                            &(index as i64 + self.0[index])
                        }
                    }
                    impl LongValues for DeltaLongValuesWrapper {}

                    segment_to_global_ords.as_mut_slice()[i] = Box::new(DeltaLongValuesWrapper(deltas));
                }
            }
        }

        Ok(Self {
            owner,
            value_count,
            global_ord_deltas,
            first_segments,
            segment_to_global_ords,
            segment_map,
        })
    }
}

pub struct OrdinalMapBaseBuilder<'a> {
    owner: Option<CacheKey>,
    acceptable_overhead_ratio: Option<f32>,
    sorted_doc_values: Option<&'a mut [Pin<Box<dyn SortedDocValues + 'a>>]>,
    sorted_set_doc_values: Option<&'a mut [Pin<Box<dyn SortedSetDocValues + 'a>>]>,
    subs_and_weights: Option<(&'a [Pin<Box<dyn TermsEnum + 'a>>], Vec<i64>)>,
}

impl<'a> Default for OrdinalMapBaseBuilder<'a> {
    fn default() -> Self {
        Self {
            owner: None,
            acceptable_overhead_ratio: None,
            sorted_doc_values: None,
            sorted_set_doc_values: None,
            subs_and_weights: None,
        }
    }
}

impl<'a> OrdinalMapBaseBuilder<'a> {
    pub fn owner(&mut self, owner: CacheKey) -> &mut Self {
        self.owner = Some(owner);
        self
    }

    pub fn acceptable_overhead_ratio(&mut self, acceptable_overhead_ratio: f32) -> &mut Self {
        self.acceptable_overhead_ratio = Some(acceptable_overhead_ratio);
        self
    }

    pub fn sorted_doc_values(&mut self, sorted_doc_values: &'a mut [Pin<Box<dyn SortedDocValues + 'a>>]) -> &mut Self {
        self.sorted_doc_values = Some(sorted_doc_values);
        self
    }

    pub fn sorted_set_doc_values(
        &mut self,
        sorted_set_doc_values: &'a mut [Pin<Box<dyn SortedSetDocValues + 'a>>],
    ) -> &mut Self {
        self.sorted_set_doc_values = Some(sorted_set_doc_values);
        self
    }

    pub fn subs_and_weights(&mut self, subs: &'a [Pin<Box<dyn TermsEnum + 'a>>], weights: Vec<i64>) -> &mut Self {
        self.subs_and_weights = Some((subs, weights));
        self
    }

    pub async fn build(self) -> Result<OrdinalMapBase, Box<dyn Error + Send + Sync + 'static>> {
        let Some(owner) = self.owner else {
            return Err(OrdinalMapBaseBuilderError::MissingOwner.into());
        };

        let acceptable_overhead_ratio = self.acceptable_overhead_ratio.unwrap_or(DEFAULT_ACCEPTABLE_OVERHEAD_RATIO);
        if acceptable_overhead_ratio <= 0.0 || acceptable_overhead_ratio >= 1.0 {
            return Err(OrdinalMapBaseBuilderError::InvalidOverheadRatio(acceptable_overhead_ratio).into());
        }

        match (self.sorted_doc_values, self.sorted_set_doc_values, self.subs_and_weights) {
            (Some(sorted_doc_values), None, None) => {
                build_from_sorted_doc_values(owner, acceptable_overhead_ratio, sorted_doc_values).await
            }
            (None, Some(sorted_set_doc_values), None) => {
                build_from_sorted_set_doc_values(owner, acceptable_overhead_ratio, sorted_set_doc_values).await
            }
            (None, None, Some(subs_and_weights)) => {
                build_from_subs_and_weights(
                    owner,
                    acceptable_overhead_ratio,
                    subs_and_weights.0,
                    subs_and_weights.1,
                )
                .await
            }
            (None, None, None) => Err(OrdinalMapBaseBuilderError::MissingValues.into()),
            _ => Err(OrdinalMapBaseBuilderError::MultipleValues.into()),
        }
    }
}

#[derive(Debug)]
pub enum OrdinalMapBaseBuilderError {
    InconsistentSubsAndWeightsLength,
    InvalidOverheadRatio(f32),
    MissingOwner,
    MissingValues,
    MultipleValues,
}

impl Display for OrdinalMapBaseBuilderError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::InconsistentSubsAndWeightsLength => f.write_str("Subs and weights must have the same length"),
            Self::InvalidOverheadRatio(overhead_ratio) => {
                write!(f, "Invalid overhead ratio; must be in the range (0.0, 1.0), exclusive: {overhead_ratio}")
            }
            Self::MissingOwner => f.write_str("Missing owner field"),
            Self::MissingValues => f.write_str("Missing sorted_doc_values, sorted_set_doc_values, or subs_and_weights"),
            Self::MultipleValues => f.write_str("Multiple values provided"),
        }
    }
}

impl Error for OrdinalMapBaseBuilderError {}

/// Create an ordinal map that uses the number of unique values of each [SortedDocValues] instance as a weight.
/// Corresponds to the Java `build(IndexReader.CacheKey, SortedDocValues[], float)` method.
async fn build_from_sorted_doc_values<'a>(
    owner: CacheKey,
    acceptable_overhead_ratio: f32,
    values: &'a mut [Pin<Box<dyn SortedDocValues + 'a>>],
) -> Result<OrdinalMapBase, Box<dyn Error + Send + Sync + 'static>> {
    let mut subs: Vec<Pin<Box<dyn TermsEnum + 'a>>> = Vec::with_capacity(values.len());
    let mut weights = Vec::with_capacity(values.len());

    for value in values.iter_mut() {
        let value: &'a mut Pin<Box<dyn SortedDocValues + 'a>> = value;
        let weight = value.as_ref().get_value_count().await?;
        let te = value.as_mut().terms_enum().await?;

        subs.push(Box::pin(te));
        weights.push(weight as i64);
    }

    let result = build_from_subs_and_weights(owner, acceptable_overhead_ratio, &subs, weights).await;
    result
}

/// Create an ordinal map that uses the number of unique values of each [SortedSetDocValues] instance as a weight.
/// Corresponds to the Java `build(IndexReader.CacheKey, SortedSetDocValues[], float)` method.
async fn build_from_sorted_set_doc_values<'a>(
    owner: CacheKey,
    acceptable_overhead_ratio: f32,
    values: &'a mut [Pin<Box<dyn SortedSetDocValues + 'a>>],
) -> Result<OrdinalMapBase, Box<dyn Error + Send + Sync + 'static>> {
    let subs = Vec::with_capacity(values.len());
    let weights = Vec::with_capacity(values.len());

    for value in values {
        subs.push(value.as_mut().terms_enum().await?);
        weights.push(value.as_ref().get_value_count().await? as i64);
    }

    build_from_subs_and_weights(owner, acceptable_overhead_ratio, &subs, weights).await
}

/// Creates an ordinal map that allows mapping ords to/from a merged space from `subs`.
/// Corresponds to the Java `build(IndexReader.CacheKey, TermsEnum[], long[], float)` method.
async fn build_from_subs_and_weights<'a>(
    owner: CacheKey,
    acceptable_overhead_ratio: f32,
    subs: &'a [Pin<Box<dyn TermsEnum + 'a>>],
    weights: Vec<i64>,
) -> Result<OrdinalMapBase, Box<dyn Error + Send + Sync + 'static>> {
    if subs.len() != weights.len() {
        return Err(OrdinalMapBaseBuilderError::InconsistentSubsAndWeightsLength.into());
    }

    // enums are not sorted, so let's sort to save memory.
    let segment_map = SegmentMap::new(&weights);
    OrdinalMapBase::from_segment_map(owner, subs, segment_map, acceptable_overhead_ratio).await
}

