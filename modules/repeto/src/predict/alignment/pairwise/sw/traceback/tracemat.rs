use std::ops::Range;

use super::{Trace, TracedAlignment, Tracer};
use super::super::super::{AlignmentOp, AlignmentStep};

struct RunningTrace {
    pub op: AlignmentOp,
    pub len: usize,
}

impl RunningTrace {
    pub fn new(op: AlignmentOp, len: usize) -> Self {
        Self { op, len }
    }

    pub fn save(self, saveto: &mut Vec<AlignmentStep>) {
        let tail = self.len % (u8::MAX as usize);
        if tail > 0 {
            saveto.push(AlignmentStep { op: self.op, len: tail as u8 });
        }
        for _ in 0..(self.len / (u8::MAX as usize)) {
            saveto.push(AlignmentStep { op: self.op, len: u8::MAX });
        }
    }
}

pub struct TraceMatrix {
    mat: Vec<Trace>,
    rows: usize,
    cols: usize,
}

impl TraceMatrix {
    pub fn new() -> Self {
        Self {
            mat: Vec::new(),
            rows: 0,
            cols: 0,
        }
    }
}

impl Tracer for TraceMatrix {
    fn reset(&mut self, rows: usize, cols: usize) {
        self.rows = rows + 1;
        self.cols = cols + 1;

        self.mat.clear();
        self.mat.resize(self.rows * self.cols, Trace::None);
    }

    #[inline(always)]
    fn gap_row(&mut self, row: usize, col: usize) {
        self.mat[(row + 1) * self.cols + (col + 1)] = Trace::GapRow;
    }

    #[inline(always)]
    fn gap_col(&mut self, row: usize, col: usize) {
        self.mat[(row + 1) * self.cols + (col + 1)] = Trace::GapCol;
    }

    #[inline(always)]
    fn equivalent(&mut self, row: usize, col: usize) {
        self.mat[(row + 1) * self.cols + (col + 1)] = Trace::Equivalent;
    }

    fn trace(&self, row: usize, col: usize) -> Result<TracedAlignment, ()> {
        let (seq1end, seq2end) = (row + 1, col + 1);
        if seq1end >= self.rows || seq2end >= self.cols {
            return Err(());
        }

        let (mut row, mut col) = (seq1end, seq2end);
        let seed = match self.mat[row * self.cols + col].try_into() {
            Err(()) => return Err(()),
            Ok(op) => op
        };

        let mut result = Vec::new();
        let mut trace = RunningTrace::new(seed, 0);

        loop {
            let op = self.mat[row * self.cols + col];
            let aop = match op.try_into() {
                Err(()) => {
                    trace.save(&mut result);
                    break;
                }
                Ok(op) => op
            };

            if aop == trace.op {
                trace.len += 1;
            } else {
                trace.save(&mut result);
                trace = RunningTrace::new(aop, 1);
            }

            match op {
                Trace::None => {
                    debug_assert!(false, "Must be unreachable!");
                    break;
                }
                Trace::GapRow => {
                    row -= 1;
                }
                Trace::GapCol => {
                    col -= 1;
                }
                Trace::Equivalent => {
                    row -= 1;
                    col -= 1;
                }
            };
        }
        let mut seq1range = Range {
            start: row,
            end: seq1end,
        };
        let mut seq2range = Range {
            start: col,
            end: seq2end,
        };
        for x in [&mut seq1range, &mut seq2range] {
            if x.start == x.end {
                x.start -= 1;
                x.end -= 1;
            }
        }
        result.reverse();

        return Ok(TracedAlignment {
            ops: result,
            seq1range,
            seq2range,
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::test_suite;

    #[test]
    fn test() {
        let mut tracer = TraceMatrix::new();
        test_suite::run_all(&mut tracer);
    }
}