//! Unit tests for SpanDeX.

#[cfg(test)]
use crate::units::{Mm, Pt, Sp};

#[test]
fn convert_mm_to_sp() {
    let expected_result = Sp(3_263_190);
    let size_in_mm = Mm(17.5);
    let cast_value = Sp::from(size_in_mm);
    assert_eq!(cast_value, expected_result);
}

#[test]
fn convert_pt_to_sp() {
    let expected_result = Sp(3_723_756);
    let size_in_pt = Pt(56.82);
    let cast_from_sp = Sp::from(size_in_pt);
    assert_eq!(cast_from_sp, expected_result);
}

#[test]
fn convert_sp_to_mm() {
    let expected_result = Mm(17.5);
    let size_in_sp = Sp(3_263_189);
    let cast_from_sp: Mm = size_in_sp.into();
    assert_eq!(cast_from_sp, expected_result);
}

#[test]
fn convert_sp_to_pt() {
    let expected_result = Pt(20.25);
    let size_in_sp = Sp(1_327_104);
    let cast_from_sp: Pt = size_in_sp.into();
    assert_eq!(cast_from_sp, expected_result);
}

#[test]
fn convert_pt_to_mm() {
    let expected_result = Mm(4.481);
    let size_in_pt = Pt(12.75);
    let cast_from_pt = Mm::from(size_in_pt);
    assert_eq!(cast_from_pt, expected_result);
}

#[test]
fn convert_mm_to_pt() {
    let expected_result = Pt(12.75);
    let size_in_mm = Mm(4.481);
    let cast_from_mm: Pt = size_in_mm.into();
    assert_eq!(cast_from_mm, expected_result);
}
