use {lucene_core::index::*, std::{collections::HashSet, path::PathBuf}, test_log::test};

#[test]
fn read_rfc_database() {
    let mut db_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    db_dir.push("tests");
    db_dir.push("rfc-database");
    let si = SegmentIndex::fs_open(&db_dir).unwrap();
    assert_eq!(si.get_version(), 28);
    assert_eq!(si.get_generation(), 1);
    assert_eq!(si.get_last_generation(), 1);
    assert_eq!(si.get_lucene_version().major(), 9);
    assert_eq!(si.get_lucene_version().minor(), 5);
    assert_eq!(si.get_lucene_version().bugfix(), 0);
    assert_eq!(si.get_id().to_string().as_str(), "0e4f01f9665661c1754333c97632152e");
    assert!(si.get_user_data().is_empty());

    let mut seen_scis = vec![false, false, false];
    const SEGMENT_IDS: [&str; 3] =
        ["0e4f01f9665661c1754333c976321509", "0e4f01f9665661c1754333c97632152a", "0e4f01f9665661c1754333c97632152d"];
    const SEGMENT_NAMES: [&str; 3] = ["_0", "_b", "_c"];
    const TIMESTAMPS: [&str; 3] = ["1676593179395", "1676593196078", "1676593196110"];
    const FILES_0: [&str; 3] = ["_0.cfe", "_0.si", "_0.cfs"];
    const FILES_1: [&str; 3] = ["_b.cfe", "_b.si", "_b.cfs"];
    const FILES_2: [&str; 12] = ["_c.fdm", "_c.si", "_c.fdt", "_c_Lucene90_0.tip", "_c_Lucene90_0.pos", "_c.nvd", "_c.fdx", "_c_Lucene90_0.doc", "_c_Lucene90_0.tim", "_c_Lucene90_0.tmd", "_c.nvm", "_c.fnm"];
    const FILES: [&[&str]; 3] = [&FILES_0, &FILES_1, &FILES_2];

    for sci in si.get_segments() {
        let seg_info = sci.get_segment_info();

        let segment_id = match seg_info.get_max_doc() {
            701 => 0,
            572 => 1,
            7885 => 2,
            _ => panic!("Unexpected segment info: max_doc={}", sci.get_segment_info().get_max_doc()),
        };
        seen_scis.as_mut_slice()[segment_id] = true;

        assert_eq!(sci.get_id().unwrap().to_string().as_str(), SEGMENT_IDS[segment_id]);
        assert_eq!(sci.get_del_count(), 0);
        assert_eq!(sci.get_soft_del_count(), 0);
        assert!(sci.get_del_gen().is_none());
        assert!(sci.get_field_infos_gen().is_none());
        assert!(sci.get_doc_values_gen().is_none());
        assert_eq!(sci.get_next_write_del_gen(), 1);
        assert_eq!(sci.get_next_write_field_infos_gen(), 1);
        assert_eq!(sci.get_next_write_doc_values_gen(), 1);
        let min_version = sci.get_min_version().unwrap();
        assert_eq!(min_version.major(), 9);
        assert_eq!(min_version.minor(), 5);
        assert_eq!(min_version.bugfix(), 0);
        let version = sci.get_version();
        assert_eq!(version.major(), 9);
        assert_eq!(version.minor(), 5);
        assert_eq!(version.bugfix(), 0);
        assert!(seg_info.get_index_sort().is_none());
        assert_eq!(seg_info.get_name(), SEGMENT_NAMES[segment_id]);
        let attributes = seg_info.get_attributes();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes.get("Lucene90StoredFieldsFormat.mode").map(|s| s.as_str()), Some("BEST_SPEED"));
        let diagnostics = seg_info.get_diagnostics();
        if segment_id < 2 {
            assert_eq!(diagnostics.len(), 8);
        } else {
            assert_eq!(diagnostics.len(), 10);
        }
        assert_eq!(diagnostics.get("java.runtime.version").map(|s| s.as_str()), Some("17.0.6+10-jvmci-22.3-b13"));
        assert_eq!(diagnostics.get("java.vendor").map(|s| s.as_str()), Some("GraalVM Community"));
        assert_eq!(diagnostics.get("lucene.version").map(|s| s.as_str()), Some("9.5.0"));
        assert_eq!(diagnostics.get("os").map(|s| s.as_str()), Some("Mac OS X"));
        assert_eq!(diagnostics.get("os.version").map(|s| s.as_str()), Some("13.1"));
        assert_eq!(diagnostics.get("os.arch").map(|s| s.as_str()), Some("aarch64"));
        assert_eq!(diagnostics.get("timestamp").map(|s| s.as_str()), Some(TIMESTAMPS[segment_id]));
        if segment_id < 2 {
            assert_eq!(diagnostics.get("source").map(|s| s.as_str()), Some("flush"));
        } else {
            assert_eq!(diagnostics.get("source").map(|s| s.as_str()), Some("merge"));
            assert_eq!(diagnostics.get("mergeFactor").map(|s| s.as_str()), Some("10"));
            assert_eq!(diagnostics.get("mergeMaxNumSegments").map(|s| s.as_str()), Some("-1"));
        }

        let expected_files: HashSet<String> = HashSet::from_iter(FILES[segment_id].iter().map(|s| s.to_string()));
        assert_eq!(seg_info.get_files(), &expected_files);
    }
}
