use std::{
    cmp::Ordering,
    env::var,
    fs::File,
    io::{Result as IoResult, Write},
    path::PathBuf,
    process::ExitCode,
};

const MAX_ITERATIONS: usize = 256;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error generating sloppy math sin/cos tables: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> IoResult<()> {
    generate_sloppy_math_sin_cos_tables()?;
    for i in 3..=24 {
        generate_bulk_operation_packed(i)?;
    }
    Ok(())
}

#[allow(non_snake_case)]
fn generate_sloppy_math_sin_cos_tables() -> IoResult<()> {
    const SIN_COS_TABS_SIZE: usize = (1 << 11) + 1;
    const SIN_COS_PI_INDEX: usize = (SIN_COS_TABS_SIZE - 1) / 2;
    const SIN_COS_PI_MUL_2_INDEX: usize = 2 * SIN_COS_PI_INDEX;
    const SIN_COS_PI_MUL_0_5_INDEX: usize = SIN_COS_PI_INDEX / 2;
    const SIN_COS_PI_MUL_1_5_INDEX: usize = 3 * SIN_COS_PI_INDEX / 2;
    // 1.57079632673412561417e+00 first 33 bits of pi/2
    let PIO2_HI: f64 = f64::from_bits(0x3FF921FB54400000);
    // 6.07710050650619224932e-11 pi/2 - PIO2_HI
    let PIO2_LO: f64 = f64::from_bits(0x3DD0B4611A626331);
    let TWOPI_HI: f64 = 4.0 * PIO2_HI;
    let TWOPI_LO: f64 = 4.0 * PIO2_LO;
    let SIN_COS_DELTA_HI: f64 = TWOPI_HI / (SIN_COS_TABS_SIZE - 1) as f64;
    let SIN_COS_DELTA_LO: f64 = TWOPI_LO / (SIN_COS_TABS_SIZE - 1) as f64;

    let mut sinTab = [0.0; SIN_COS_TABS_SIZE];
    let mut cosTab = [0.0; SIN_COS_TABS_SIZE];

    for i in 0..SIN_COS_TABS_SIZE {
        // angle: in [0,2*PI].
        let angle = i as f64 * SIN_COS_DELTA_HI + i as f64 * SIN_COS_DELTA_LO;
        let mut sinAngle = angle.sin();
        let mut cosAngle = angle.cos();

        // For indexes corresponding to null cosine or sine, we make sure the value is zero
        // and not an epsilon. This allows for a much better accuracy for results close to zero.
        if i == SIN_COS_PI_INDEX || i == SIN_COS_PI_MUL_2_INDEX {
            sinAngle = 0.0;
        } else if i == SIN_COS_PI_MUL_0_5_INDEX || i == SIN_COS_PI_MUL_1_5_INDEX {
            cosAngle = 0.0;
        }

        sinTab[i] = sinAngle;
        cosTab[i] = cosAngle;
    }

    // asin
    let ASIN_MAX_VALUE_FOR_TABS: f64 = 73.0_f64.to_radians().sin();
    const ASIN_TABS_SIZE: usize = (1 << 13) + 1;
    let ASIN_DELTA: f64 = ASIN_MAX_VALUE_FOR_TABS / (ASIN_TABS_SIZE - 1) as f64;
    const ONE_DIV_F2: f64 = 1.0 / 2.0;
    const ONE_DIV_F3: f64 = 1.0 / 6.0;
    const ONE_DIV_F4: f64 = 1.0 / 24.0;

    let mut asinTab = [0.0; ASIN_TABS_SIZE];
    let mut asinDer1DivF1Tab = [0.0; ASIN_TABS_SIZE];
    let mut asinDer2DivF2Tab = [0.0; ASIN_TABS_SIZE];
    let mut asinDer3DivF3Tab = [0.0; ASIN_TABS_SIZE];
    let mut asinDer4DivF4Tab = [0.0; ASIN_TABS_SIZE];

    for i in 0..ASIN_TABS_SIZE {
        // x: in [0,ASIN_MAX_VALUE_FOR_TABS].
        let x = i as f64 * ASIN_DELTA;
        asinTab[i] = x.asin();
        let oneMinusXSqInv = 1.0 / (1.0 - x * x);
        let oneMinusXSqInv0_5 = oneMinusXSqInv.sqrt();
        let oneMinusXSqInv1_5 = oneMinusXSqInv0_5 * oneMinusXSqInv;
        let oneMinusXSqInv2_5 = oneMinusXSqInv1_5 * oneMinusXSqInv;
        let oneMinusXSqInv3_5 = oneMinusXSqInv2_5 * oneMinusXSqInv;

        asinDer1DivF1Tab[i] = oneMinusXSqInv0_5;
        asinDer2DivF2Tab[i] = (x * oneMinusXSqInv1_5) * ONE_DIV_F2;
        asinDer3DivF3Tab[i] = ((1.0 + 2.0 * x * x) * oneMinusXSqInv2_5) * ONE_DIV_F3;
        asinDer4DivF4Tab[i] = ((5.0 + 2.0 * x * (2.0 + x * (5.0 - 2.0 * x))) * oneMinusXSqInv3_5) * ONE_DIV_F4;
    }

    let mut table_filename = PathBuf::from(var("OUT_DIR").unwrap());
    table_filename.push("sloppy_math_sin_cos_tables.rs");
    let mut f = File::create(&table_filename)?;

    write_f64_table(&mut f, "SIN_TAB", &sinTab)?;
    write_f64_table(&mut f, "COS_TAB", &cosTab)?;
    write_f64_table(&mut f, "ASIN_TAB", &asinTab)?;
    write_f64_table(&mut f, "ASIN_DER1_DIV_F1_TAB", &asinDer1DivF1Tab)?;
    write_f64_table(&mut f, "ASIN_DER2_DIV_F2_TAB", &asinDer2DivF2Tab)?;
    write_f64_table(&mut f, "ASIN_DER3_DIV_F3_TAB", &asinDer3DivF3Tab)?;
    write_f64_table(&mut f, "ASIN_DER4_DIV_F4_TAB", &asinDer4DivF4Tab)?;
    Ok(())
}

fn write_f64_table<W: Write>(w: &mut W, table_name: &str, c: &[f64]) -> IoResult<()> {
    writeln!(w, "const {table_name}: [f64; {}] = [", c.len())?;

    for el in c.iter() {
        let el_long = el.to_bits();
        writeln!(w, "    unsafe {{ ::std::mem::transmute::<u64, f64>({el_long}) }},")?;
    }
    writeln!(w, "];")?;
    writeln!(w)?;
    Ok(())
}

fn generate_bulk_operation_packed(bits_per_value: usize) -> IoResult<()> {
    let mut table_filename = PathBuf::from(var("OUT_DIR").unwrap());
    table_filename.push(format!("bulk_operation_packed_{bits_per_value}.rs"));
    let mut f = File::create(&table_filename)?;

    writeln!(f, "impl Decoder for BulkOperationPacked<{bits_per_value}> {{")?;
    write!(
        f,
        r#"        fn long_block_count(&self) -> usize {{
        self.long_block_count
    }}

    fn long_value_count(&self) -> usize {{
        self.long_value_count
    }}

    fn byte_block_count(&self) -> usize {{
        self.byte_block_count
    }}

    fn byte_value_count(&self) -> usize {{
        self.byte_value_count
    }}
"#
    )?;

    generate_bulk_operation_packed_decode_u64(&mut f, bits_per_value, 32)?;
    generate_bulk_operation_packed_decode_u64(&mut f, bits_per_value, 64)?;
    generate_bulk_operation_packed_decode_u8(&mut f, bits_per_value, 32)?;
    generate_bulk_operation_packed_decode_u8(&mut f, bits_per_value, 64)?;

    writeln!(f, "}}")?;
    writeln!(f)?;
    writeln!(f, "bulk_operation_packed_default_encode!({bits_per_value});")?;
    writeln!(f)?;
    writeln!(f, "impl BulkOperation for BulkOperationPacked<{bits_per_value}> {{")?;
    writeln!(f, "    bulk_operation_packed_basic_methods!();")?;
    writeln!(f, "}}")?;
    Ok(())
}

fn generate_bulk_operation_packed_decode_u64<W>(w: &mut W, bits_per_value: usize, otype: usize) -> IoResult<()>
where
    W: Write,
{
    let mask = (1 << bits_per_value) - 1;

    writeln!(w)?;
    writeln!(w, "    fn decode_u64_to_i{otype}(&mut self, blocks: &[u64], values: &mut [i{otype}], iterations: usize) -> IoResult<()> {{")?;
    writeln!(w, "        let mut blocks_offset = 0;")?;
    writeln!(w, "        let mut values_offset = 0;")?;
    writeln!(w, "        for _ in 0..iterations {{")?;
    writeln!(w, "            let block = blocks[blocks_offset];")?;
    writeln!(w, "            blocks_offset += 1;")?;

    let mut shift: i64 = 64 - bits_per_value as i64;
    for i in 0..=MAX_ITERATIONS {
        if i == MAX_ITERATIONS {
            panic!("Too many iterations for generate_bulk_operation_packed_decode_u64(bits_per_value={bits_per_value} and i{otype})");
        }
        assert!(shift >= 0);

        writeln!(w, "            values[values_offset] = ((block >> {shift}) & {mask}) as i{otype};")?;
        writeln!(w, "            values_offset += 1;")?;

        shift -= bits_per_value as i64;
        if shift == 0 {
            break;
        }

        if shift > 0 {
            continue;
        }

        // Need to merge current block with next block
        let lshift = (-shift) as usize;
        shift += 64;

        writeln!(w, "            let prev_block = block;")?;
        writeln!(w, "            let block = blocks[blocks_offset];")?;
        writeln!(w, "            blocks_offset += 1;")?;
        writeln!(w, "            values[values_offset] = (((prev_block << {lshift}) & {mask}) | (block >> {shift})) as i{otype};")?;
        writeln!(w, "            values_offset += 1;")?;
    }

    writeln!(w, "        }}")?;
    writeln!(w)?;
    writeln!(w, "        Ok(())")?;
    writeln!(w, "    }}")?;
    Ok(())
}

fn generate_bulk_operation_packed_decode_u8<W>(w: &mut W, bits_per_value: usize, otype: usize) -> IoResult<()>
where
    W: Write,
{
    if bits_per_value >= 8 {
        generate_bulk_operation_packed_decode_u8_multibyte(w, bits_per_value, otype)
    } else {
        generate_bulk_operation_packed_decode_u8_singlebyte(w, bits_per_value, otype)
    }
}

fn generate_bulk_operation_packed_decode_u8_singlebyte<W>(
    w: &mut W,
    bits_per_value: usize,
    otype: usize,
) -> IoResult<()>
where
    W: Write,
{
    let mask = (1 << bits_per_value) - 1;

    writeln!(w)?;
    writeln!(w, "    fn decode_u8_to_i{otype}(&mut self, blocks: &[u8], values: &mut [i{otype}], iterations: usize) -> IoResult<()> {{")?;
    writeln!(w, "        let mut blocks_offset = 0;")?;
    writeln!(w, "        let mut values_offset = 0;")?;
    writeln!(w, "        for _ in 0..iterations {{")?;
    writeln!(w, "            let block = blocks[blocks_offset];")?;
    writeln!(w, "            blocks_offset += 1;")?;

    let mut shift: i64 = 8 - bits_per_value as i64;
    for i in 0..=MAX_ITERATIONS {
        if i == MAX_ITERATIONS {
            panic!("Too many iterations for generate_bulk_operation_packed_decode_u8_singlebyte(bits_per_value={bits_per_value}, otype={otype})");
        }

        writeln!(w, "            values[values_offset] = ((block >> {shift}) & {mask}) as i{otype};")?;
        writeln!(w, "            values_offset += 1;")?;

        shift -= bits_per_value as i64;
        if shift == 0 {
            break;
        }

        if shift > 0 {
            continue;
        }

        // Need to merge current block with next block
        let lshift = (-shift) as usize;
        shift += 8;

        writeln!(w, "            let prev_block = block;")?;
        writeln!(w, "            let block = blocks[blocks_offset];")?;
        writeln!(w, "            blocks_offset += 1;")?;
        writeln!(w, "            values[values_offset] = (((prev_block << {lshift}) & {mask}) | (block >> {shift})) as i{otype};")?;
        writeln!(w, "            values_offset += 1;")?;
    }

    writeln!(w, "        }}")?;
    writeln!(w)?;
    writeln!(w, "    Ok(())")?;
    writeln!(w, "    }}")?;
    Ok(())
}

fn generate_bulk_operation_packed_decode_u8_multibyte<W>(w: &mut W, bits_per_value: usize, otype: usize) -> IoResult<()>
where
    W: Write,
{
    writeln!(w)?;
    writeln!(w, "    fn decode_u8_to_i{otype}(&mut self, blocks: &[u8], values: &mut [i{otype}], iterations: usize) -> IoResult<()> {{")?;
    writeln!(w, "        let mut blocks_offset = 0;")?;
    writeln!(w, "        let mut values_offset = 0;")?;
    writeln!(w, "        for _ in 0..iterations {{")?;

    let mut shift: i64 = bits_per_value as i64;
    let mut unwritten = vec![];
    let mut byte_index = 0;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations == MAX_ITERATIONS {
            panic!("Too many iterations for generate_bulk_operation_packed_decode_u8_multibyte(bits_per_value={bits_per_value}, otype={otype})");
        }

        shift -= 8;
        writeln!(w, "            // shift = {shift}")?;
        writeln!(w, "            let byte{byte_index} = blocks[blocks_offset];")?;
        writeln!(w, "            blocks_offset += 1;")?;
        unwritten.push((byte_index, shift));
        byte_index += 1;

        if shift <= 0 {
            // Write the bytes to values.
            write!(w, "            values[values_offset] = ")?;
            let mut first = true;

            for (byte_index, shift) in unwritten.iter() {
                if !first {
                    write!(w, " | ")?;
                } else {
                    first = false;
                }

                match shift.cmp(&0) {
                    Ordering::Less => {
                        let rshift = -shift;
                        write!(w, "(byte{byte_index} as i{otype} >> {rshift})")?;
                    }
                    Ordering::Equal => {
                        write!(w, "(byte{byte_index} as i{otype})")?;
                    }
                    _ => {
                        write!(w, "((byte{byte_index} as i{otype}) << {shift})")?;
                    }
                }
            }
            writeln!(w, ";")?;
            writeln!(w, "            values_offset += 1;")?;
            unwritten.clear();

            if shift != 0 {
                shift += bits_per_value as i64;
                unwritten.push((byte_index, shift));
            } else if shift == 0 {
                break;
            }
        }
    }

    assert!(unwritten.is_empty());
    writeln!(w, "        }}")?;
    writeln!(w)?;
    writeln!(w, "    Ok(())")?;
    writeln!(w, "    }}")?;

    Ok(())
}
