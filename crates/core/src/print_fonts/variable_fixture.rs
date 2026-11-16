//! Test-only in-memory variation tables around the existing licensed Hack face.
//! No changed font file is bundled or written. OpenType fvar/gvar/tuple formats:
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats
//! At wght=700, A's outline moves +100 units and its advance grows +500 units.
//! This intentionally minimal fixture tests instance propagation, not a font
//! production/export tool or a claim of full variable-font conformance.
use skrifa::{FontRef, MetadataProvider};
use std::collections::BTreeMap;

fn u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}
fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}
fn push16(data: &mut Vec<u8>, value: u16) {
    data.extend(value.to_be_bytes());
}
fn push32(data: &mut Vec<u8>, value: u32) {
    data.extend(value.to_be_bytes());
}
fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, chunk| {
        let mut bytes = [0; 4];
        bytes[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(bytes))
    })
}

pub(crate) fn variable_hack() -> Vec<u8> {
    let original = epaint_default_fonts::HACK_REGULAR;
    let font = FontRef::new(original).unwrap();
    let glyph = font.charmap().map('A').unwrap().to_u32() as usize;
    let mut tables = BTreeMap::<[u8; 4], Vec<u8>>::new();
    for i in 0..usize::from(u16_at(original, 4)) {
        let record = 12 + 16 * i;
        let start = u32_at(original, record + 8) as usize;
        let len = u32_at(original, record + 12) as usize;
        tables.insert(
            original[record..record + 4].try_into().unwrap(),
            original[start..start + len].to_vec(),
        );
    }
    let glyph_count = usize::from(u16_at(&tables[b"maxp"], 4));
    let offset = if u16_at(&tables[b"head"], 50) == 0 {
        usize::from(u16_at(&tables[b"loca"], 2 * glyph)) * 2
    } else {
        u32_at(&tables[b"loca"], 4 * glyph) as usize
    };
    let contours = usize::from(u16_at(&tables[b"glyf"], offset));
    assert!((1..64).contains(&contours), "A must be a simple glyph");
    let points = usize::from(u16_at(&tables[b"glyf"], offset + 10 + 2 * (contours - 1))) + 1;

    let mut fvar = Vec::new();
    for value in [1, 0, 16, 2, 1, 20, 0, 8] {
        push16(&mut fvar, value);
    }
    fvar.extend(b"wght");
    for value in [400, 400, 700] {
        push32(&mut fvar, value << 16);
    }
    push16(&mut fvar, 0);
    push16(&mut fvar, 256);
    tables.insert(*b"fvar", fvar);

    // Private point-number list zero means all outline and four phantom points.
    let mut deltas = vec![0];
    for _ in 0..points {
        deltas.extend([0, 100]);
    } // one signed-byte X delta
    deltas.push(0x80); // left phantom unchanged
    deltas.push(0x40); // right phantom: one signed-word X delta
    push16(&mut deltas, 500);
    deltas.extend([0x80, 0x80]); // vertical phantoms unchanged
    deltas.extend(std::iter::repeat_n(0x80, points + 4)); // all Y deltas zero
    let mut tuple = Vec::new();
    for value in [1, 10, deltas.len() as u16, 0xa000, 0x4000] {
        push16(&mut tuple, value);
    }
    tuple.extend(deltas);
    let mut gvar = Vec::new();
    for value in [1, 0, 1, 0] {
        push16(&mut gvar, value);
    }
    push32(&mut gvar, 20); // non-null offset for the empty shared-tuple array
    push16(&mut gvar, glyph_count as u16);
    push16(&mut gvar, 1); // long offsets
    push32(&mut gvar, (20 + (glyph_count + 1) * 4) as u32);
    for i in 0..=glyph_count {
        push32(&mut gvar, if i <= glyph { 0 } else { tuple.len() as u32 });
    }
    gvar.extend(tuple);
    tables.insert(*b"gvar", gvar);

    // Rebuild the SFNT directory and checksums; all other test-face tables stay
    // unchanged. Readers in this test do not rely on an invalid checksum.
    tables.get_mut(b"head").unwrap()[8..12].fill(0);
    let n = tables.len() as u16;
    let power = 1u16 << n.ilog2();
    let mut result = Vec::new();
    push32(&mut result, 0x0001_0000);
    for value in [n, power * 16, n.ilog2() as u16, n * 16 - power * 16] {
        push16(&mut result, value);
    }
    result.resize(12 + 16 * usize::from(n), 0);
    let mut head_start = 0;
    for (i, (tag, data)) in tables.iter().enumerate() {
        result.resize(result.len().next_multiple_of(4), 0);
        let start = result.len();
        let record = 12 + 16 * i;
        result[record..record + 4].copy_from_slice(tag);
        result[record + 4..record + 8].copy_from_slice(&checksum(data).to_be_bytes());
        result[record + 8..record + 12].copy_from_slice(&(start as u32).to_be_bytes());
        result[record + 12..record + 16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        result.extend(data);
        if tag == b"head" {
            head_start = start;
        }
    }
    let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&result));
    result[head_start + 8..head_start + 12].copy_from_slice(&adjustment.to_be_bytes());
    result
}
