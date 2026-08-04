//! Source: `Compiler/src/CostModel.cpp:50-101`

#[allow(non_snake_case)]
fn parallelAddSat(a: u64, b: u64) -> u64 {
    let mut res = a.wrapping_add(b);
    let overflow = ((res & 0x8080808080808080) | ((a & b) & 0x8080808080808080))
        | ((a | b) & !res & 0x8080808080808080);
    res |= (overflow << 1).wrapping_sub(overflow >> 7);
    res | overflow
}

#[allow(non_snake_case)]
fn parallelMulSat(a: u64, b: i32) -> u64 {
    if b <= 0 {
        return 0;
    }
    if b == 1 {
        return a;
    }
    let mut res = 0u64;
    let mut va = a;
    let mut vb = b;
    while vb > 0 {
        if vb & 1 != 0 {
            res = parallelAddSat(res, va);
        }
        va = parallelAddSat(va, va);
        vb >>= 1;
    }
    res
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cost {
    pub(crate) model: u64,
    pub(crate) constant: u64,
}

impl Cost {
    pub const kLiteral: u64 = !0u64;

    pub fn new(cost: i32, constant: u64) -> Self {
        Self {
            model: if cost < 0x7f { cost as u64 } else { 0x7f },
            constant,
        }
    }

    pub fn add(&self, other: &Cost) -> Cost {
        let result: Cost = Cost {
            model: parallelAddSat(self.model, other.model),
            constant: 0,
        };
        result
    }

    pub fn add_assign(&mut self, other: &Cost) {
        self.model = parallelAddSat(self.model, other.model);
        self.constant = 0;
    }

    pub fn mul(&self, other: i32) -> Cost {
        let result: Cost = Cost {
            model: parallelMulSat(self.model, other),
            constant: 0,
        };
        result
    }

    pub fn fold(x: &Cost, y: &Cost) -> Cost {
        let new_model = parallelAddSat(x.model, y.model);
        let newconstant: u64 = x.constant & y.constant;

        let extra = if newconstant == Self::kLiteral {
            0
        } else {
            1 | (0x0101010101010101u64 & newconstant)
        };

        let result: Cost = Cost {
            model: parallelAddSat(new_model, extra),
            constant: newconstant,
        };
        result
    }
}
