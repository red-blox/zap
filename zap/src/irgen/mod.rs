#![allow(clippy::should_implement_trait)]
use std::{cell::RefCell, collections::HashMap, fmt::Display, iter, rc::Rc};

use crate::config::{NumTy, Range, Ty};

pub mod des;
pub mod ser;

pub trait Gen {
	fn buf(&self) -> &OutputBuffer;
	fn generate<'a, 'src: 'a, I>(self, names: &[String], types: I) -> Vec<Stmt>
	where
		I: Iterator<Item = &'a Ty<'src>>;

	fn get_var_occurrences(&mut self) -> &mut HashMap<String, usize>;
	fn add_occurrence(&mut self, name: &str) -> (String, Expr) {
		match self.get_var_occurrences().get(name) {
			Some(occurrences) => {
				let occurrences_inc = occurrences + 1;
				self.get_var_occurrences().insert(name.into(), occurrences_inc);
				let suffixed_name = format!("{name}_{occurrences_inc}");
				(suffixed_name.clone(), Expr::from(suffixed_name.as_str()))
			}
			None => {
				self.get_var_occurrences().insert(name.into(), 1);
				let suffixed_name = format!("{name}_1");
				(suffixed_name.clone(), Expr::from(suffixed_name.as_str()))
			}
		}
	}

	fn scopes(&mut self) -> &mut Scopes;

	fn get_bitpack(&mut self) -> (BitpackMask, Var) {
		if let Some(existing) = self
			.scopes()
			.dynamic()
			.bitpack_budget
			.last_mut()
			.filter(|(shift, _)| *shift < (BitpackMask::BITS as u8 - 1))
		{
			existing.0 += 1;
			(1 << existing.0, Var::Name(existing.1.clone()))
		} else {
			let (name, _) = self.add_occurrence("bool");
			self.scopes().dynamic().bitpack_budget.push((0, name.clone()));
			(1, Var::Name(name))
		}
	}

	fn variant_storage(&mut self, amount: usize) -> VariantStorageKind {
		if amount == 1 {
			VariantStorageKind::None
			// 0 is variant 1, 1 is variant 2
		} else if amount == 2 {
			VariantStorageKind::Bit(self.get_bitpack())
		} else if self.scopes().dynamic().remaining_bitpack_budget() as usize >= amount {
			VariantStorageKind::Bitpack(iter::repeat_with(|| self.get_bitpack()).take(amount).collect())
		} else {
			VariantStorageKind::Full(NumTy::from_f64(0.0, amount as f64 - 1.0), amount)
		}
	}

	fn push_writestring(&mut self, expr: Expr, range: &Range, count: Expr) {
		self.buf().clone().push_writestring(self.scopes(), expr, range, count);
	}

	fn push_write_copy(&mut self, expr: Expr, range: &Range, count: Expr) {
		self.buf().clone().push_write_copy(self.scopes(), expr, range, count);
	}

	fn alloc_dynamic(&mut self, range: &Range, count: Expr) {
		self.buf().clone().alloc_dynamic(self.scopes(), range, count);
	}

	fn readvector3(&mut self) -> impl FnOnce() -> Expr + 'static + use<Self> {
		readvector3(self.scopes())
	}

	fn readvector(
		&mut self,
		x_numty: NumTy,
		y_numty: NumTy,
		z_numty: Option<NumTy>,
	) -> impl FnOnce() -> Expr + 'static + use<Self> {
		readvector(self.scopes(), x_numty, y_numty, z_numty)
	}
}

enum OutputEntryKind {
	Stmt(Stmt),
	LazyStmt(Box<dyn FnOnce() -> Stmt>),
	Buffer(OutputBuffer),
}

pub struct OutputEntry(OutputEntryKind);

impl From<Stmt> for OutputEntry {
	fn from(value: Stmt) -> Self {
		OutputEntry(OutputEntryKind::Stmt(value))
	}
}

impl<F: FnOnce() -> Stmt + 'static> From<F> for OutputEntry {
	fn from(value: F) -> Self {
		OutputEntry(OutputEntryKind::LazyStmt(Box::new(value)))
	}
}

impl From<OutputBuffer> for OutputEntry {
	fn from(value: OutputBuffer) -> Self {
		OutputEntry(OutputEntryKind::Buffer(value))
	}
}

pub fn alloc(expr: Expr) -> Expr {
	Var::from("alloc").call(vec![expr])
}

pub fn read(expr: Expr) -> Expr {
	Var::from("read").call(vec![expr])
}

#[derive(Default, Clone)]
pub struct OutputBuffer(Rc<RefCell<Vec<OutputEntry>>>);

impl OutputBuffer {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push<T: Into<OutputEntry>>(&self, item: T) {
		self.0.borrow_mut().push(item.into());
	}

	pub fn output(self) -> Vec<Stmt> {
		let mut output = vec![];

		for entry in self.0.take() {
			match entry.0 {
				OutputEntryKind::Stmt(stmt) => output.push(stmt),
				OutputEntryKind::LazyStmt(make) => output.push(make()),
				OutputEntryKind::Buffer(buf) => output.extend(buf.output()),
			}
		}

		output
	}

	pub fn push_alloc(&self, expr: Expr) {
		self.push(Stmt::Call(Var::from("alloc"), None, vec![expr]));
	}

	pub fn alloc_dynamic(&self, scopes: &mut Scopes, range: &Range, mut count: Expr) {
		let scope = scopes.alloc();

		if let Some(exact) = range.exact() {
			scope.size += exact as usize;
		} else {
			if let Some(min) = range.min() {
				scope.size += min as usize;
				count = count.sub(min.into());
			}

			if let Some(max) = range.max() {
				scope.max.add(max as usize);
			}

			self.push_alloc(count);
		}
	}

	pub fn push_local(&self, name: String, expr: Option<Expr>) {
		self.push(Stmt::Local(name, expr))
	}

	pub fn push_assign(&self, var: Var, expr: Expr) {
		self.push(Stmt::Assign(var, expr))
	}

	pub fn push_assert(&self, expr: Expr, msg: String) {
		self.push(Stmt::Assert(expr, msg))
	}

	pub fn push_writestring(&mut self, scopes: &mut Scopes, expr: Expr, range: &Range, count: Expr) {
		self.alloc_dynamic(scopes, range, count.clone());

		self.push(Stmt::Call(
			Var::from("buffer").nindex("writestring"),
			None,
			vec!["outgoing_buff".into(), "outgoing_apos".into(), expr, count],
		));
	}

	pub fn push_write_copy(&mut self, scopes: &mut Scopes, expr: Expr, range: &Range, count: Expr) {
		self.alloc_dynamic(scopes, range, count.clone());

		self.push(Stmt::Call(
			Var::from("buffer").nindex("copy"),
			None,
			vec!["outgoing_buff".into(), "outgoing_apos".into(), expr, 0.0.into(), count],
		));
	}

	pub fn push_read_copy(&mut self, into: Var, count: Expr) {
		self.push_assign(
			into.clone(),
			Var::from("buffer").nindex("create").call(vec![count.clone()]),
		);

		self.push(Stmt::Call(
			Var::from("buffer").nindex("copy"),
			None,
			vec![
				into.into(),
				0.0.into(),
				"incoming_buff".into(),
				read(count.clone()),
				count,
			],
		));
	}

	pub fn push_range_check(&self, expr: Expr, range: Range) {
		if let Some(min) = range.min() {
			self.push_assert(expr.clone().gte(min.into()), format!("value is less than {min}!"))
		}

		if let Some(max) = range.max() {
			self.push_assert(expr.clone().lte(max.into()), format!("value is more than {max}!"))
		}
	}

	pub fn push_utf8_check(&self, expr: Expr) {
		self.push_assert(
			Var::NameIndex(Var::Name("utf8".into()).into(), "len".to_string())
				.call(vec![expr])
				.neq(Expr::Nil),
			"value is not valid utf-8".into(),
		);
	}
}

macro_rules! numbers {
	($($ty:ident),+) => {
		paste::paste! {
			impl OutputBuffer {
				$(
					fn [<push_write $ty _raw>](&mut self, scopes: &mut Scopes, expr: Expr, immediate: bool) {
						let scope = scopes.alloc();
						let (offset, dyn_offset) = scope.alloc(NumTy::[<$ty:upper>].size());
						let cursor_var = scope.cursor_var.clone();

						let make = move || {
							let dyn_offset = (*dyn_offset.borrow()).clone();
							let mut offset_expr = Expr::Num((if immediate { 0 } else { offset } + dyn_offset.offset) as f64);
							if let Some(expr) = dyn_offset.expr {
								offset_expr = offset_expr.add(expr);
							}

							Stmt::Call(
								Var::from("buffer").nindex(concat!("write", stringify!($ty))),
								None,
								vec![
									"outgoing_buff".into(),
									Expr::Var(Var::Name(cursor_var).into()).add(offset_expr),
									expr,
								],
							)
						};

						if immediate {
							self.push(make());
						} else {
							self.push(make);
						}
					}

					fn [<push_write $ty>](&mut self, scopes: &mut Scopes, expr: Expr) {
						self.[<push_write $ty _raw>](scopes, expr, false);
					}
				)+

				// fn push_writenumty(&mut self, scopes: &mut Scopes, expr: Expr, numty: NumTy) {
				// 	match numty {
				// 		$(
				// 			NumTy::[<$ty:upper>] => self.[<push_write $ty>](scopes, expr)
				// 		),+
				// 	}
				// }
			}

			$(
				pub fn [<read $ty _raw>](scopes: &mut Scopes, immediate: bool) -> impl FnOnce() -> Expr + 'static + use<> {
					let scope = scopes.alloc();
					let (offset, dyn_offset) = scope.alloc(NumTy::[<$ty:upper>].size());
					let cursor_var = scope.cursor_var.clone();

					let make = move || {
						let dyn_offset = (*dyn_offset.borrow()).clone();
						let mut offset_expr = Expr::Num((if immediate { 0 } else { offset } + dyn_offset.offset) as f64);
						if let Some(expr) = dyn_offset.expr {
							offset_expr = offset_expr.add(expr);
						}

						Var::from("buffer")
							.nindex(concat!("read", stringify!($ty)))
							.call(vec!["incoming_buff".into(), Expr::Var(Var::Name(cursor_var).into()).add(offset_expr)])
					};

					let (value, make) = if immediate { (Some(make()), None) } else { (None, Some(make)) };

					move || {
						match (value, make) {
							(Some(val), _) => val,
							(_, Some(make)) => make(),
							_ => unreachable!()
						}
					}
				}

				pub fn [<read $ty>](scopes: &mut Scopes) -> impl FnOnce() -> Expr + 'static + use<> {
					[<read $ty _raw>](scopes, false)
				}
			)+

			pub fn readnumty_raw(scopes: &mut Scopes, numty: NumTy, immediate: bool) -> Box<dyn FnOnce() -> Expr + 'static> {
				match numty {
					$(
						NumTy::[<$ty:upper>] => Box::new([<read $ty _raw>](scopes, immediate))
					),+
				}
			}

			pub fn readnumty(scopes: &mut Scopes, numty: NumTy) -> Box<dyn FnOnce() -> Expr + 'static> {
				readnumty_raw(scopes, numty, false)
			}

			pub trait GenNumExt: Gen {
				$(
					fn [<push_write $ty _raw>](&mut self, expr: Expr, immediate: bool) {
						self.buf().clone().[<push_write $ty _raw>](&mut self.scopes(), expr, immediate);
					}

					fn [<push_write $ty>](&mut self, expr: Expr) {
						self.buf().clone().[<push_write $ty>](&mut self.scopes(), expr);
					}

					fn [<read $ty _raw>](&mut self, immediate: bool) -> impl FnOnce() -> Expr + 'static + use<Self> {
						[<read $ty _raw>](self.scopes(), immediate)
					}

					fn [<read $ty>](&mut self) -> impl FnOnce() -> Expr + 'static + use<Self> {
						self.[<read $ty _raw>](false)
					}
				)+

				fn push_writenumty_raw(&mut self, expr: Expr, numty: NumTy, immediate: bool) {
					match numty {
						$(
							NumTy::[<$ty:upper>] => self.buf().clone().[<push_write $ty _raw>](&mut self.scopes(), expr, immediate)
						),+
					}
				}

				fn push_writenumty(&mut self, expr: Expr, numty: NumTy) {
					match numty {
						$(
							NumTy::[<$ty:upper>] => self.buf().clone().[<push_write $ty>](&mut self.scopes(), expr)
						),+
					}
				}

				fn readnumty_raw(&mut self, numty: NumTy, immediate: bool) -> Box<dyn FnOnce() -> Expr + 'static> {
					match numty {
						$(
							NumTy::[<$ty:upper>] => Box::new([<read $ty _raw>](&mut self.scopes(), immediate))
						),+
					}
				}

				fn readnumty(&mut self, numty: NumTy) -> Box<dyn FnOnce() -> Expr + 'static> {
					self.readnumty_raw(numty, false)
				}
			}

			impl<T: Gen> GenNumExt for T {}
		}
	};
}

numbers!(f32, f64, u8, u16, u32, i8, i16, i32);

pub fn readstring(count: Expr) -> Expr {
	Var::from("buffer")
		.nindex("readstring")
		.call(vec!["incoming_buff".into(), read(count.clone()), count])
}

pub fn readvector3(scopes: &mut Scopes) -> impl FnOnce() -> Expr + 'static + use<> {
	let x = readf32(scopes);
	let y = readf32(scopes);
	let z = readf32(scopes);

	move || Expr::Vector3(Box::new(x()), Box::new(y()), Box::new(z()))
}

pub fn readvector(
	scopes: &mut Scopes,
	x_numty: NumTy,
	y_numty: NumTy,
	z_numty: Option<NumTy>,
) -> impl FnOnce() -> Expr + 'static + use<> {
	let x = readnumty(scopes, x_numty);
	let y = readnumty(scopes, y_numty);
	let z = z_numty.map(|z_numty| readnumty(scopes, z_numty));

	move || Expr::Vector(Box::new(x()), Box::new(y()), z.map(|z| Box::new(z())))
}

pub type BitpackMask = u16;

pub struct DynamicScope {
	pub bitpack_budget: Vec<(u8, String)>,
	pub buf: OutputBuffer,
}

impl DynamicScope {
	pub fn remaining_bitpack_budget(&self) -> u8 {
		self.bitpack_budget
			.last()
			.map(|(shift, _)| BitpackMask::BITS as u8 - (shift + 1))
			.unwrap_or(BitpackMask::BITS as u8)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum AllocMax {
	Unbounded,
	Unknown,
	Max(usize),
}

impl AllocMax {
	pub fn add(&mut self, max: usize) {
		match self {
			AllocMax::Unbounded => {}
			AllocMax::Unknown => *self = AllocMax::Max(max),
			AllocMax::Max(present) => *present += max,
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct LazyAllocData {
	pub offset: usize,
	pub expr: Option<Expr>,
}

#[must_use]
pub struct AllocScope {
	pub size: usize,
	pub multiplier: usize,
	pub max: AllocMax,
	pub buf: OutputBuffer,
	pub cursor_var: String,
	pub offset: Rc<RefCell<LazyAllocData>>,
}

impl AllocScope {
	pub fn new(buf: OutputBuffer, cursor_var: String) -> Self {
		Self {
			size: 0,
			multiplier: 1,
			max: AllocMax::Unknown,
			buf,
			cursor_var,
			offset: Rc::new(RefCell::new(LazyAllocData::default())),
		}
	}

	#[must_use]
	pub fn alloc(&mut self, size: usize) -> (usize, Rc<RefCell<LazyAllocData>>) {
		let offset = self.size;
		self.size += size;
		(offset, self.offset.clone())
	}

	pub fn offset_by(&self, offset: usize) {
		self.offset.borrow_mut().offset += offset;
	}

	pub fn offset_expr(&self, expr: Expr) {
		self.offset.borrow_mut().expr = Some(expr);
	}
}

#[derive(Default)]
pub struct Scopes {
	pub dynamic: Vec<DynamicScope>,
	pub alloc: Vec<AllocScope>,
}

impl Scopes {
	pub fn dynamic(&mut self) -> &mut DynamicScope {
		self.dynamic.last_mut().unwrap()
	}

	pub fn alloc(&mut self) -> &mut AllocScope {
		self.alloc.last_mut().unwrap()
	}
}

#[derive(Debug)]
pub enum VariantStorageKind {
	Full(NumTy, usize),
	Bitpack(Vec<(BitpackMask, Var)>),
	Bit((BitpackMask, Var)),
	None,
}

#[derive(Debug, Clone)]
pub enum Stmt {
	Local(String, Option<Expr>),
	LocalTuple(Vec<String>, Option<Expr>),
	Assign(Var, Expr),
	Error(String),
	Assert(Expr, String),

	Call(Var, Option<String>, Vec<Expr>),

	NumFor { var: String, from: Expr, to: Expr },
	GenFor { key: String, val: String, obj: Expr },
	If(Expr),
	ElseIf(Expr),
	Else,

	End,
}

#[derive(Debug, Clone)]
pub enum Var {
	Name(String),

	NameIndex(Box<Var>, String),
	ExprIndex(Box<Var>, Box<Expr>),
}

impl Var {
	pub fn nindex(self, index: impl Into<String>) -> Self {
		Self::NameIndex(Box::new(self), index.into())
	}

	pub fn eindex(self, index: Expr) -> Self {
		Self::ExprIndex(Box::new(self), Box::new(index))
	}

	pub fn call(self, args: Vec<Expr>) -> Expr {
		Expr::Call(Box::new(self), None, args)
	}
}

impl From<&str> for Var {
	fn from(name: &str) -> Self {
		Self::Name(name.into())
	}
}

impl Display for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Name(name) => write!(f, "{name}"),
			Self::NameIndex(var, index) => write!(f, "{var}.{index}"),
			Self::ExprIndex(var, index) => write!(f, "{var}[{index}]"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	// Keyword Literals
	False,
	True,
	Nil,

	// Literals
	Str(String),
	StrOrBool(String),
	Var(Box<Var>),
	Num(f64),
	BinaryNum(BitpackMask),

	// Function Call
	Call(Box<Var>, Option<String>, Vec<Expr>),

	// Table
	Table(Box<Vec<(Expr, Expr)>>),

	// Datatypes
	Color3(Box<Expr>, Box<Expr>, Box<Expr>),
	Vector3(Box<Expr>, Box<Expr>, Box<Expr>),
	Vector(Box<Expr>, Box<Expr>, Option<Box<Expr>>),

	// Unary Operators
	Len(Box<Expr>),
	Not(Box<Expr>),

	// Boolean Binary Operators
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	// Comparison Binary Operators
	Gte(Box<Expr>, Box<Expr>),
	Lte(Box<Expr>, Box<Expr>),
	Neq(Box<Expr>, Box<Expr>),
	Gt(Box<Expr>, Box<Expr>),
	Lt(Box<Expr>, Box<Expr>),
	Eq(Box<Expr>, Box<Expr>),

	// Arithmetic Binary Operators
	Add(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
}

impl Expr {
	pub fn len(self) -> Self {
		Self::Len(Box::new(self))
	}

	pub fn not(self) -> Self {
		Self::Not(Box::new(self))
	}

	pub fn and(self, other: Self) -> Self {
		Self::And(Box::new(self), Box::new(other))
	}

	pub fn or(self, other: Self) -> Self {
		Self::Or(Box::new(self), Box::new(other))
	}

	pub fn gte(self, other: Self) -> Self {
		Self::Gte(Box::new(self), Box::new(other))
	}

	pub fn lte(self, other: Self) -> Self {
		Self::Lte(Box::new(self), Box::new(other))
	}

	pub fn neq(self, other: Self) -> Self {
		Self::Neq(Box::new(self), Box::new(other))
	}

	pub fn gt(self, other: Self) -> Self {
		Self::Gt(Box::new(self), Box::new(other))
	}

	pub fn lt(self, other: Self) -> Self {
		Self::Lt(Box::new(self), Box::new(other))
	}

	pub fn eq(self, other: Self) -> Self {
		Self::Eq(Box::new(self), Box::new(other))
	}

	pub fn add(self, other: Self) -> Self {
		Self::Add(Box::new(self), Box::new(other))
	}

	pub fn sub(self, other: Self) -> Self {
		Self::Sub(Box::new(self), Box::new(other))
	}

	pub fn mul(self, other: Self) -> Self {
		Self::Mul(Box::new(self), Box::new(other))
	}
}

impl From<String> for Expr {
	fn from(name: String) -> Self {
		Self::Var(Box::new(Var::Name(name)))
	}
}

impl From<Var> for Expr {
	fn from(var: Var) -> Self {
		Self::Var(Box::new(var))
	}
}

impl From<&str> for Expr {
	fn from(name: &str) -> Self {
		Self::Var(Box::new(name.into()))
	}
}

impl From<f64> for Expr {
	fn from(num: f64) -> Self {
		Self::Num(num)
	}
}

impl Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::False => write!(f, "false"),
			Self::True => write!(f, "true"),
			Self::Nil => write!(f, "nil"),

			Self::Str(string) => write!(f, "\"{string}\""),
			Self::StrOrBool(string) => {
				if string == "false" || string == "true" {
					write!(f, "{string}")
				} else {
					write!(f, "\"{string}\"")
				}
			}

			Self::Var(var) => write!(f, "{var}"),
			Self::Num(num) => write!(f, "{num}"),
			Self::BinaryNum(num) => write!(f, "{num:#018b}"),

			Self::Call(var, method, args) => match method {
				Some(method) => write!(
					f,
					"{}:{}({})",
					var,
					method,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),

				None => write!(
					f,
					"{}({})",
					var,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),
			},

			Self::Table(items) => write!(
				f,
				"{{ {} }}",
				items
					.iter()
					.map(|(key, value)| format!("[{key}] = {value}"))
					.collect::<Vec<_>>()
					.join(", ")
			),

			Self::Color3(r, g, b) => write!(f, "Color3.fromRGB({r}, {g}, {b})"),
			Self::Vector3(x, y, z) => write!(f, "Vector3.new({x}, {y}, {z})"),
			Self::Vector(x, y, z) => write!(
				f,
				"vector.create({}, {}, {})",
				x,
				y,
				z.as_ref().unwrap_or(&Box::new(Expr::Num(0 as f64)))
			),

			Self::Len(expr) => write!(f, "#{expr}"),
			Self::Not(expr) => write!(f, "not {expr}"),

			Self::And(lhs, rhs) => write!(f, "{lhs} and {rhs}"),
			Self::Or(lhs, rhs) => write!(f, "{lhs} or {rhs}"),

			Self::Gte(lhs, rhs) => write!(f, "{lhs} >= {rhs}"),
			Self::Lte(lhs, rhs) => write!(f, "{lhs} <= {rhs}"),
			Self::Neq(lhs, rhs) => write!(f, "{lhs} ~= {rhs}"),
			Self::Gt(lhs, rhs) => write!(f, "{lhs} > {rhs}"),
			Self::Lt(lhs, rhs) => write!(f, "{lhs} < {rhs}"),
			Self::Eq(lhs, rhs) => write!(f, "{lhs} == {rhs}"),

			Self::Add(lhs, rhs) => write!(f, "({lhs} + {rhs})"),
			Self::Sub(lhs, rhs) => write!(f, "({lhs} - {rhs})"),
			Self::Mul(lhs, rhs) => write!(f, "({lhs} * {rhs})"),
		}
	}
}
