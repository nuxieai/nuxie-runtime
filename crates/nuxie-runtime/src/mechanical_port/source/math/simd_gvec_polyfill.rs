use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Index, IndexMut, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
    SubAssign,
};

pub type Swizzle = u32;
pub const fn pack_swizzle2(source_length: u32, i0: u32, i1: u32) -> Swizzle {
    (i1 << 5) | (i0 << 3) | source_length
}
pub const fn pack_swizzle3(source_length: u32, i0: u32, i1: u32, i2: u32) -> Swizzle {
    (i2 << 7) | pack_swizzle2(source_length, i0, i1)
}
pub const fn pack_swizzle4(source_length: u32, i0: u32, i1: u32, i2: u32, i3: u32) -> Swizzle {
    (i3 << 9) | pack_swizzle3(source_length, i0, i1, i2)
}
pub const fn unpack_swizzle_source_vector_length(swizzle: Swizzle) -> u32 {
    swizzle & 7
}
pub const fn unpack_swizzle_index(swizzle: Swizzle, index: u32) -> u32 {
    (swizzle >> (index * 2 + 3)) & 3
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct GVec<T, const N: usize> {
    pub data: [T; N],
}
impl<T: Copy + Default, const N: usize> Default for GVec<T, N> {
    fn default() -> Self {
        Self {
            data: [T::default(); N],
        }
    }
}
impl<T: Copy, const N: usize> GVec<T, N> {
    pub const fn from_array(data: [T; N]) -> Self {
        Self { data }
    }
    pub fn splat(value: T) -> Self {
        Self { data: [value; N] }
    }
    pub fn swizzle<const M: usize>(&self, indices: [usize; M]) -> GVec<T, M> {
        GVec {
            data: core::array::from_fn(|i| self.data[indices[i]]),
        }
    }
}
impl<T, const N: usize> Index<usize> for GVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}
impl<T, const N: usize> IndexMut<usize> for GVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}

impl<T: Copy> GVec<T, 1> {
    pub fn x(self) -> T {
        self[0]
    }
    pub fn r(self) -> T {
        self[0]
    }
}
impl<T: Copy> GVec<T, 2> {
    pub fn x(self) -> T {
        self[0]
    }
    pub fn y(self) -> T {
        self[1]
    }
    pub fn r(self) -> T {
        self[0]
    }
    pub fn g(self) -> T {
        self[1]
    }
    pub fn yx(self) -> Self {
        self.swizzle([1, 0])
    }
    pub fn xyxy(self) -> GVec<T, 4> {
        self.swizzle([0, 1, 0, 1])
    }
    pub fn yxyx(self) -> GVec<T, 4> {
        self.swizzle([1, 0, 1, 0])
    }
    pub fn xxyy(self) -> GVec<T, 4> {
        self.swizzle([0, 0, 1, 1])
    }
    pub fn yyxx(self) -> GVec<T, 4> {
        self.swizzle([1, 1, 0, 0])
    }
}
impl<T: Copy> GVec<T, 3> {
    pub fn x(self) -> T {
        self[0]
    }
    pub fn y(self) -> T {
        self[1]
    }
    pub fn z(self) -> T {
        self[2]
    }
    pub fn xyz(self) -> Self {
        self
    }
}
impl<T: Copy> GVec<T, 4> {
    pub fn x(self) -> T {
        self[0]
    }
    pub fn y(self) -> T {
        self[1]
    }
    pub fn z(self) -> T {
        self[2]
    }
    pub fn w(self) -> T {
        self[3]
    }
    pub fn xy(self) -> GVec<T, 2> {
        self.swizzle([0, 1])
    }
    pub fn zw(self) -> GVec<T, 2> {
        self.swizzle([2, 3])
    }
    pub fn xyz(self) -> GVec<T, 3> {
        self.swizzle([0, 1, 2])
    }
    pub fn yxwz(self) -> Self {
        self.swizzle([1, 0, 3, 2])
    }
    pub fn zwxy(self) -> Self {
        self.swizzle([2, 3, 0, 1])
    }
    pub fn zyxw(self) -> Self {
        self.swizzle([2, 1, 0, 3])
    }
    pub fn xwzy(self) -> Self {
        self.swizzle([0, 3, 2, 1])
    }
    pub fn xzyw(self) -> Self {
        self.swizzle([0, 2, 1, 3])
    }
    pub fn www(self) -> GVec<T, 3> {
        self.swizzle([3, 3, 3])
    }
}

macro_rules! vector_binary { ($trait:ident,$method:ident,$assign_trait:ident,$assign_method:ident,$op:tt) => {
    impl<T:Copy+$trait<Output=T>,const N:usize> $trait for GVec<T,N>{type Output=Self;fn $method(self,rhs:Self)->Self{Self{data:core::array::from_fn(|i|self[i] $op rhs[i])}}}
    impl<T:Copy+$trait<Output=T>,const N:usize> $trait<T> for GVec<T,N>{type Output=Self;fn $method(self,rhs:T)->Self{Self{data:core::array::from_fn(|i|self[i] $op rhs)}}}
    impl<T:Copy+$assign_trait,const N:usize> $assign_trait for GVec<T,N>{fn $assign_method(&mut self,rhs:Self){for i in 0..N{self[i].$assign_method(rhs[i]);}}}
    impl<T:Copy+$assign_trait,const N:usize> $assign_trait<T> for GVec<T,N>{fn $assign_method(&mut self,rhs:T){for i in 0..N{self[i].$assign_method(rhs);}}}
};}
vector_binary!(Add,add,AddAssign,add_assign,+);
vector_binary!(Sub,sub,SubAssign,sub_assign,-);
vector_binary!(Mul,mul,MulAssign,mul_assign,*);
vector_binary!(Div,div,DivAssign,div_assign,/);
vector_binary!(Rem,rem,RemAssign,rem_assign,%);
vector_binary!(BitAnd,bitand,BitAndAssign,bitand_assign,&);
vector_binary!(BitOr,bitor,BitOrAssign,bitor_assign,|);
vector_binary!(BitXor,bitxor,BitXorAssign,bitxor_assign,^);
vector_binary!(Shl,shl,ShlAssign,shl_assign,<<);
vector_binary!(Shr,shr,ShrAssign,shr_assign,>>);
macro_rules! scalar_left_binary { ($ty:ty,$trait:ident,$method:ident,$op:tt) => { impl<const N:usize> $trait<GVec<$ty,N>> for $ty { type Output=GVec<$ty,N>; fn $method(self,rhs:GVec<$ty,N>)->Self::Output { GVec{data:core::array::from_fn(|i|self $op rhs[i])} } } }; }
macro_rules! scalar_left_for_type { ($ty:ty) => { scalar_left_binary!($ty,Add,add,+); scalar_left_binary!($ty,Sub,sub,-); scalar_left_binary!($ty,Mul,mul,*); scalar_left_binary!($ty,Div,div,/); scalar_left_binary!($ty,Rem,rem,%); scalar_left_binary!($ty,BitAnd,bitand,&); scalar_left_binary!($ty,BitOr,bitor,|); scalar_left_binary!($ty,BitXor,bitxor,^); scalar_left_binary!($ty,Shl,shl,<<); scalar_left_binary!($ty,Shr,shr,>>); }; }
scalar_left_for_type!(i8);
scalar_left_for_type!(i16);
scalar_left_for_type!(i32);
scalar_left_for_type!(i64);
scalar_left_for_type!(i128);
scalar_left_for_type!(isize);
scalar_left_for_type!(u8);
scalar_left_for_type!(u16);
scalar_left_for_type!(u32);
scalar_left_for_type!(u64);
scalar_left_for_type!(u128);
scalar_left_for_type!(usize);
scalar_left_binary!(f32,Add,add,+);
scalar_left_binary!(f32,Sub,sub,-);
scalar_left_binary!(f32,Mul,mul,*);
scalar_left_binary!(f32,Div,div,/);
scalar_left_binary!(f32,Rem,rem,%);
impl<T: Copy + Neg<Output = T>, const N: usize> Neg for GVec<T, N> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            data: core::array::from_fn(|i| -self[i]),
        }
    }
}
impl<T: Copy + Not<Output = T>, const N: usize> Not for GVec<T, N> {
    type Output = Self;
    fn not(self) -> Self {
        Self {
            data: core::array::from_fn(|i| !self[i]),
        }
    }
}

pub trait MaskElement: Copy + Default {
    fn true_mask() -> Self;
}
macro_rules! masks {($($ty:ty),*$(,)?)=>{$(impl MaskElement for $ty{fn true_mask()->Self{!0}})*};}
masks!(i8, i16, i32, i64, i128, isize);
pub fn compare<T: Copy, const N: usize, M: MaskElement>(
    a: GVec<T, N>,
    b: GVec<T, N>,
    mut predicate: impl FnMut(T, T) -> bool,
) -> GVec<M, N> {
    GVec {
        data: core::array::from_fn(|i| {
            if predicate(a[i], b[i]) {
                M::true_mask()
            } else {
                M::default()
            }
        }),
    }
}
