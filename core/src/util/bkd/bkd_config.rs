use {
    std::mem::size_of,
};

///Default maximum number of point in each leaf block
pub const DEFAULT_MAX_POINTS_IN_LEAF_NODE: usize = 512;

/// Maximum number of index dimensions (2 * max index dimensions)
pub const MAX_DIMS: usize = 16;

/// Maximum number of index dimensions
pub const MAX_INDEX_DIMS: usize = 8;

/// Basic parameters for indexing points on the BKD tree.
#[derive(Debug)]
pub struct BkdConfig {
    /// How many dimensions we are storing at the leaf (data) nodes
    num_dims: usize,

    /// How many dimensions we are indexing in the internal nodes
    num_index_dims: usize,
    
    /// How many bytes each value in each dimension takes.
    bytes_per_dim: usize,
    
    /// max points allowed on a Leaf block
    max_points_in_leaf_node: usize,
}

impl BkdConfig {
    #[inline]
    pub const fn new(num_dims: usize, num_index_dims: usize, bytes_per_dim: usize, max_points_in_leaf_node: usize) -> Self {
        Self::verify_params(num_dims, num_index_dims, bytes_per_dim, max_points_in_leaf_node);

        Self {
            num_dims,
            num_index_dims,
            bytes_per_dim,
            max_points_in_leaf_node,
        }
    }

    #[inline]
    pub const fn get_num_dims(&self) -> usize {
        self.num_dims
    }

    #[inline]
    pub const fn get_num_index_dims(&self) -> usize {
        self.num_index_dims
    }

    #[inline]
    pub const fn get_bytes_per_dim(&self) -> usize {
        self.bytes_per_dim
    }

    #[inline]
    pub const fn get_max_points_in_leaf_node(&self) -> usize {
        self.max_points_in_leaf_node
    }

    #[inline]
    pub const fn get_packed_bytes_length(&self) -> usize {
        self.num_dims * self.bytes_per_dim
    }

    #[inline]
    pub const fn get_packed_index_bytes_length(&self) -> usize {
        self.num_index_dims * self.bytes_per_dim
    }

    #[inline]
    pub const fn get_bytes_per_doc(&self) -> usize {
        self.get_packed_bytes_length() + size_of::<i32>()
    }

    const fn verify_params(
        num_dims: usize,
        num_index_dims: usize,
        bytes_per_dim: usize,
        max_points_in_leaf_node: usize,
    ) {
        if num_dims < 1 || num_dims > MAX_DIMS {
            panic!("num_dims must be between 1 and {MAX_DIMS}: {num_dims}");
        }
        
        if num_index_dims < 1 || num_index_dims > MAX_INDEX_DIMS {
            panic!("num_index_dims must be between 1 and {MAX_INDEX_DIMS}: {num_index_dims}");
        }

        if num_index_dims > num_dims {
            panic!("num_index_dims must be <= num_dims: {num_index_dims} > {num_dims}");
        }

        if bytes_per_dim == 0 {
            panic!("bytes_per_dim must be > 0: {bytes_per_dim}");
        }

        if max_points_in_leaf_node == 0 {
            panic!("max_points_in_leaf_node must be > 0: {max_points_in_leaf_node}");
        }
    }
}