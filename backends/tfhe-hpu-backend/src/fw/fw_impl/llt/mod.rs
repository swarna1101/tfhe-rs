use super::*;

pub mod kogge;
pub mod vardeg;

use super::rtl::{Rtl, VarCell};
use kogge::{Carry, RippleCarry};
use vardeg::*;

use crate::asm::iop::opcode::*;
use crate::asm::{self, OperandKind, Pbs};
use crate::fw::metavar::MetaVarCell;
use crate::fw::program::Program;
use crate::pbs_by_name;
use itertools::{EitherOrBoth, Itertools};
use std::collections::HashMap;
use tracing::{instrument, trace};

crate::impl_fw!("Llt" [
    ADD => fw_impl::llt::iop_add;
    SUB => fw_impl::llt::iop_sub;
    MUL => fw_impl::llt::iop_mul;
    DIV => fw_impl::ilp_div::iop_div;
    MOD => fw_impl::ilp_div::iop_mod;

    // NB: fallback to ilp
    // TODO: Add dedicated llt implementation
    OVF_ADD => fw_impl::ilp::iop_overflow_add;
    OVF_SUB => fw_impl::ilp::iop_overflow_sub;
    OVF_MUL => fw_impl::ilp::iop_overflow_mul;

    // NB: fallback to ilp
    // TODO: Add dedicated llt implementation
    ROT_R => fw_impl::ilp::iop_rotate_right;
    ROT_L => fw_impl::ilp::iop_rotate_left;
    SHIFT_R => fw_impl::ilp::iop_shift_right;
    SHIFT_L => fw_impl::ilp::iop_shift_left;

    ADDS => fw_impl::llt::iop_adds;
    SUBS => fw_impl::llt::iop_subs;
    SSUB => fw_impl::llt::iop_ssub;
    MULS => fw_impl::llt::iop_muls;
    DIVS => fw_impl::ilp_div::iop_divs;
    MODS => fw_impl::ilp_div::iop_mods;

    // NB: fallback to ilp
    // TODO: Add dedicated llt implementation
    ROTS_R => fw_impl::ilp::iop_rotate_scalar_right;
    ROTS_L => fw_impl::ilp::iop_rotate_scalar_left;
    SHIFTS_R => fw_impl::ilp::iop_shift_scalar_right;
    SHIFTS_L => fw_impl::ilp::iop_shift_scalar_left;

    // NB: fallback to ilp
    // TODO: Add dedicated llt implementation
    OVF_ADDS => fw_impl::ilp::iop_overflow_adds;
    OVF_SUBS => fw_impl::ilp::iop_overflow_subs;
    OVF_SSUB => fw_impl::ilp::iop_overflow_ssub;
    OVF_MULS => fw_impl::ilp::iop_overflow_muls;

    BW_AND => (|prog| {fw_impl::ilp::iop_bw(prog, asm::dop::PbsBwAnd::default().into())});
    BW_OR  => (|prog| {fw_impl::ilp::iop_bw(prog, asm::dop::PbsBwOr::default().into())});
    BW_XOR => (|prog| {fw_impl::ilp::iop_bw(prog, asm::dop::PbsBwXor::default().into())});

    CMP_GT  => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpGtMrg"), pbs_by_name!("CmpGt"))});
    CMP_GTE => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpGteMrg"), pbs_by_name!("CmpGte"))});
    CMP_LT  => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpLtMrg"), pbs_by_name!("CmpLt"))});
    CMP_LTE => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpLteMrg"), pbs_by_name!("CmpLte"))});
    CMP_EQ  => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpEqMrg"), pbs_by_name!("CmpEq"))});
    CMP_NEQ => (|prog| {fw_impl::llt::iop_cmp(prog, pbs_by_name!("CmpNeqMrg"), pbs_by_name!("CmpNeq"))});

    IF_THEN_ZERO => fw_impl::ilp::iop_if_then_zero;
    IF_THEN_ELSE => fw_impl::ilp::iop_if_then_else;

    ERC_20 => fw_impl::llt::iop_erc_20;
    MEMCPY => fw_impl::ilp::iop_memcpy;

    COUNT0 => fw_impl::ilp_log::iop_count0;
    COUNT1 => fw_impl::ilp_log::iop_count1;
    ILOG2 => fw_impl::ilp_log::iop_ilog2;
    LEAD0 => fw_impl::ilp_log::iop_lead0;
    LEAD1 => fw_impl::ilp_log::iop_lead1;
    TRAIL0 => fw_impl::ilp_log::iop_trail0;
    TRAIL1 => fw_impl::ilp_log::iop_trail1;
]);

// ----------------------------------------------------------------------------
// API
// ----------------------------------------------------------------------------

#[instrument(level = "trace", skip(prog))]
pub fn iop_add(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Src, 1);

    // Add Comment header
    prog.push_comment("ADD Operand::Dst Operand::Src Operand::Src".to_string());
    iop_addx(prog, dst, src_a, src_b);
}

pub fn iop_adds(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Imm, 0);

    // Add Comment header
    prog.push_comment("ADDS Operand::Dst Operand::Src Operand::Immediat".to_string());
    iop_addx(prog, dst, src_a, src_b);
}

#[instrument(level = "trace", skip(prog))]
pub fn iop_sub(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Src, 1);

    // Add Comment header
    prog.push_comment("SUB Operand::Dst Operand::Src Operand::Src".to_string());
    iop_subx(prog, dst, src_a, src_b);
}

pub fn iop_subs(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Imm, 0);

    // Add Comment header
    prog.push_comment("SUBS Operand::Dst Operand::Src Operand::Immediat".to_string());
    iop_subx(prog, dst, src_a, src_b);
}

pub fn iop_ssub(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Imm, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Src, 0);

    // Add Comment header
    prog.push_comment("SSUB Operand::Dst Operand::Src Operand::Immediat".to_string());
    iop_subx(prog, dst, src_a, src_b);
}

#[instrument(level = "trace", skip(prog))]
pub fn iop_cmp(prog: &mut Program, mrg_op: Pbs, cmp_op: Pbs) {
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Operand
    let src_b = prog.iop_template_var(OperandKind::Src, 1);

    // Add Comment header
    prog.push_comment(format!(
        "CMP_{cmp_op} Operand::Dst Operand::Src Operand::Src"
    ));

    // Deferred implementation to generic cmpx function
    iop_cmpx(prog, &dst[0], &src_a, &src_b, mrg_op, cmp_op).add_to_prog(prog);
}

#[instrument(level = "trace", skip(prog))]
pub fn iop_mul(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Src, 1);

    // Add Comment header
    prog.push_comment("MUL Operand::Dst Operand::Src Operand::Src".to_string());
    // Deferred implementation to generic mulx function
    iop_mulx(prog, dst, src_a, src_b).add_to_prog(prog);
}

#[instrument(level = "trace", skip(prog))]
pub fn iop_muls(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst = prog.iop_template_var(OperandKind::Dst, 0);
    // SrcA -> Operand
    let src_a = prog.iop_template_var(OperandKind::Src, 0);
    // SrcB -> Immediat
    let src_b = prog.iop_template_var(OperandKind::Imm, 0);

    // Add Comment header
    prog.push_comment("MULS Operand::Dst Operand::Src Operand::Immediat".to_string());
    // Deferred implementation to generic mulx function
    iop_mulx(prog, dst, src_a, src_b).add_to_prog(prog);
}

/// Implement erc_20 fund xfer
/// Targeted algorithm is as follow:
/// 1. Check that from has enough funds
/// 2. Compute real_amount to xfer (i.e. amount or 0)
/// 3. Compute new amount (from - new_amount, to + new_amount)
#[instrument(level = "trace", skip(prog))]
pub fn iop_erc_20(prog: &mut Program) {
    // Allocate metavariables:
    // Dest -> Operand
    let dst_from = prog.iop_template_var(OperandKind::Dst, 0);
    let dst_to = prog.iop_template_var(OperandKind::Dst, 1);
    // Src -> Operand
    let src_from = prog.iop_template_var(OperandKind::Src, 0);
    let src_to = prog.iop_template_var(OperandKind::Src, 1);
    // Src Amount -> Operand
    let src_amount = prog.iop_template_var(OperandKind::Src, 2);

    // Add Comment header
    prog.push_comment("ERC_20 (new_from, new_to) <- (from, to, amount)".to_string());

    // TODO: Make this a parameter or sweep this
    // All these little parameters would be very handy to write an
    // exploration/compilation program which would try to minimize latency by
    // playing with these.
    let kogge_blk_w = 10;
    let ripple = true;

    let tree = {
        let props = prog.params();
        let tfhe_params: asm::DigitParameters = props.clone().into();
        let lut = pbs_by_name!("IfFalseZeroed");
        let dst_to = VarCell::from_vec(dst_to);
        let dst_from = VarCell::from_vec(dst_from);
        let src_to = VarCell::from_vec(src_to);
        let src_from = VarCell::from_vec(src_from);
        let src_amount = VarCell::from_vec(src_amount);

        // Check if from has enough funds
        let enough_fund = iop_cmpx_rtl(
            prog,
            src_from.clone(),
            src_amount.clone(),
            pbs_by_name!("CmpGteMrg"),
            pbs_by_name!("CmpGte"),
        );

        let src_amount = src_amount
            .into_iter()
            .map(|x| {
                x.mac(tfhe_params.msg_range(), &enough_fund)
                    .pbs(&lut)
                    .into_iter()
                    .next()
                    .unwrap()
            })
            .collect::<Vec<_>>();

        if ripple {
            kogge::ripple_add(dst_to, src_to, src_amount.clone(), None)
                + kogge::ripple_sub(prog, dst_from, src_from, src_amount)
        } else {
            kogge::add(prog, dst_to, src_to, src_amount.clone(), None, kogge_blk_w)
                + kogge::sub(prog, dst_from, src_from, src_amount, kogge_blk_w)
        }
    };
    tree.add_to_prog(prog);
}

// ----------------------------------------------------------------------------
// Helper Functions
// ----------------------------------------------------------------------------
fn iop_addx(
    prog: &mut Program,
    dst: Vec<MetaVarCell>,
    src_a: Vec<MetaVarCell>,
    src_b: Vec<MetaVarCell>,
) {
    {
        // Convert MetaVarCell in VarCell for Rtl analysis
        let a = VarCell::from_vec(src_a);
        let b = VarCell::from_vec(src_b);
        // Do a + b with the kogge stone adder
        kogge::cached_add(prog, a, b, None, dst)
    } // Any reference to any metavar not linked to the RTL is dropped here
    .add_to_prog(prog);
}

fn iop_subx(
    prog: &mut Program,
    dst: Vec<MetaVarCell>,
    src_a: Vec<MetaVarCell>,
    src_b: Vec<MetaVarCell>,
) {
    {
        // Convert MetaVarCell in VarCell for Rtl analysis
        let a = VarCell::from_vec(src_a);
        let b = VarCell::from_vec(src_b);
        let b_inv = bw_inv(prog, b);
        let one = Carry::Ripple(RippleCarry(VarCell::from(prog.new_imm(1))));
        kogge::cached_add(prog, a, b_inv, Some(one), dst)
    }
    .add_to_prog(prog);
}

/// Generic mul operation
/// One destination and two sources operation
/// Source could be Operand or Immediat
#[instrument(level = "trace", skip(prog))]
pub fn iop_mulx(
    prog: &mut Program,
    dst: Vec<metavar::MetaVarCell>,
    src_a: Vec<metavar::MetaVarCell>,
    src_b: Vec<metavar::MetaVarCell>,
) -> Rtl {
    let props = prog.params();
    let tfhe_params: asm::DigitParameters = props.clone().into();
    let blk_w = props.blk_w();

    // Transform metavars into RTL vars
    let mut dst = VarCell::from_vec(dst);
    let src_a = VarCell::from_vec(src_a);
    let src_b = VarCell::from_vec(src_b);
    let max_deg = VarDeg {
        deg: props.max_val(),
        nu: props.nu,
    };

    let pbs_msg = pbs_by_name!("MsgOnly");
    let pbs_carry = pbs_by_name!("CarryInMsg");
    let pbs_mul_lsb = pbs_by_name!("MultCarryMsgLsb");
    let pbs_mul_msb = pbs_by_name!("MultCarryMsgMsb");
    let max_carry = (props.max_msg() * props.max_msg()) >> props.msg_w;

    let mut mul_map: HashMap<usize, Vec<VarCellDeg>> = HashMap::new();
    itertools::iproduct!(0..blk_w, 0..blk_w).for_each(|(i, j)| {
        let pp = src_a[i].mac(tfhe_params.msg_range(), &src_b[j]);
        let lsb = pp.single_pbs(&pbs_mul_lsb);
        let msb = pp.single_pbs(&pbs_mul_msb);
        mul_map
            .entry(i + j)
            .or_default()
            .push(VarCellDeg::new(max_carry, lsb));
        mul_map
            .entry(i + j + 1)
            .or_default()
            .push(VarCellDeg::new(max_carry, msb));
    });

    for (blk, dst) in dst.iter_mut().enumerate() {
        let mut to_sum: VecVarCellDeg = mul_map.remove(&blk).unwrap().into();
        let mut bootstrap = |sum: &VarCellDeg| -> VarCellDeg {
            trace!(target: "llt:mulx:bootstrap", "bootstrap: {:?}", sum);
            if sum.deg.deg > props.max_msg() {
                mul_map.entry(blk + 1).or_default().push(VarCellDeg::new(
                    sum.deg.deg >> props.msg_w,
                    sum.var.single_pbs(&pbs_carry),
                ));
            }
            VarCellDeg::new(props.max_msg(), sum.var.single_pbs(&pbs_msg))
        };

        while to_sum.len() > 1 {
            let prev_len = to_sum.len();

            to_sum = to_sum
                .deg_chunks(&max_deg)
                // Leveled Sum
                .map(|mut chunk| {
                    trace!(target: "ilp:mulx", "leveled chunk: {:?}", chunk);

                    while chunk.len() > 1 {
                        chunk = chunk
                            .chunks(2)
                            .map(|chunk| match chunk.len() {
                                1 => chunk[0].clone(),
                                2 => &chunk[0] + &chunk[1],
                                _ => panic!("Invalid chunk size"),
                            })
                            .collect()
                    }

                    chunk.into_iter().next().unwrap()
                })
                // Bootstrap
                .map(|sum| {
                    assert!(sum.deg.nu <= props.nu);
                    if sum.deg == max_deg {
                        bootstrap(&sum)
                    } else {
                        sum
                    }
                })
                .collect::<Vec<_>>()
                .into();

            // If no element has been bootstrapped, bootstrap the worst case
            // This will be very unlikely, but if it ever happened it would have hanged
            // the whole loop. Also, the output needs to be bootstrapped,
            // anyway.
            (to_sum.0.iter().all(|x| x.deg.nu > 1) || prev_len == to_sum.len()).then(|| {
                let max = to_sum.max_mut().unwrap();
                *max = bootstrap(max);
            });
        }

        let out = to_sum.first().unwrap();

        assert!(
            {
                let deg = out.deg.clone();
                deg.deg <= props.max_msg() && deg.nu == 1
            },
            "Output variable {blk} is not bootstrapped"
        );

        *dst <<= &out.var;
    }

    Rtl::from(dst)
}

/// Generic Cmp operation
/// One destination block and two sources operands
/// Source could be Operand or Immediat
#[instrument(level = "trace", skip(prog))]
pub fn iop_cmpx(
    prog: &mut Program,
    dst: &metavar::MetaVarCell,
    src_a: &[metavar::MetaVarCell],
    src_b: &[metavar::MetaVarCell],
    mrg_lut: Pbs,
    cmp_lut: Pbs,
) -> Rtl {
    let mut dst = VarCell::from(dst);
    let src_a = src_a.iter().map(VarCell::from).collect();
    let src_b = src_b.iter().map(VarCell::from).collect();
    dst <<= &iop_cmpx_rtl(prog, src_a, src_b, mrg_lut, cmp_lut);
    Rtl::from(vec![dst])
}

/// Generic Cmp operation
/// One destination block and two sources operands
/// Source could be Operand or Immediat
#[instrument(level = "trace", skip(prog))]
pub fn iop_cmpx_rtl(
    prog: &mut Program,
    src_a: Vec<VarCell>,
    src_b: Vec<VarCell>,
    mrg_lut: Pbs,
    cmp_lut: Pbs,
) -> VarCell {
    let props = prog.params();
    let tfhe_params: asm::DigitParameters = props.clone().into();

    // Wrapped given cmp_op and comp_sign lookup table in MetaVar
    let pbs_none = pbs_by_name!("None");
    let cmp_sign = pbs_by_name!("CmpSign");
    let cmp_reduce = pbs_by_name!("CmpReduce");

    // Pack A and B elements by pairs
    let packed = std::iter::zip(src_a.chunks(2), src_b.chunks(2))
        .map(|(a, b)| {
            let pack_a = if a.len() > 1 {
                // Reset noise for future block merge through sub
                a[0].mac(tfhe_params.msg_range(), &a[1])
                    .single_pbs(&pbs_none)
            } else {
                a[0].clone()
            };

            let pack_b = if b.len() > 1 {
                b[0].mac(tfhe_params.msg_range(), &b[1])
                    .single_pbs(&pbs_none)
            } else {
                b[0].clone()
            };
            (pack_a, pack_b)
        })
        .collect::<Vec<_>>();

    let mut merged = packed
        .into_iter()
        .map(|(a, b)| &(&a - &b).single_pbs(&cmp_sign) + 1)
        .collect::<Vec<_>>();

    while merged.len() > 2 {
        merged = merged
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut chunk| {
                let left = chunk.next();
                let right = chunk.next();
                match (left, right) {
                    (Some(l), None) => l,
                    (Some(l), Some(r)) => {
                        l.mac(tfhe_params.msg_range(), &r).single_pbs(&cmp_reduce)
                    }
                    _ => panic!("Chunk misbehaved"),
                }
            })
            .collect()
    }

    match merged.len() {
        2 => merged[0]
            .mac(tfhe_params.msg_range(), &merged[1])
            .single_pbs(&mrg_lut),
        1 => merged[0].single_pbs(&cmp_lut),
        _ => panic!("Fix your bugs!"),
    }
}

fn bw_inv(prog: &mut Program, b: Vec<VarCell>) -> Vec<VarCell> {
    let blk_w = prog.params().blk_w();
    let imm = (0..blk_w).map(|_| VarCell::from(prog.new_imm((1 << prog.params().msg_w) - 1)));
    b.iter()
        .zip_longest(imm)
        .map(|r| match r {
            EitherOrBoth::Right(i) => i,
            EitherOrBoth::Both(b, i) => &i - b,
            EitherOrBoth::Left(_) => {
                panic!(
                    "The input to be inverted is greater than blk_w({}): {}",
                    blk_w,
                    b.len()
                )
            }
        })
        .collect::<Vec<_>>()
}
