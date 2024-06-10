use crate::{
	ast::{primitive::AstNumber, range::AstRange},
	hir::range::HirRange,
	meta::Report,
};

use super::HirBuilder;

impl<'a> HirBuilder<'a> {
	fn unwrap_astrange(&mut self, ast: AstRange) -> (Option<AstNumber>, Option<AstNumber>) {
		let ast_span = ast.span();
		let min_max = match ast {
			AstRange::WithMinMax(_, min, max) => (Some(min), Some(max)),
			AstRange::WithMax(_, max) => (None, Some(max)),
			AstRange::WithMin(_, min) => (Some(min), None),
			AstRange::Exact(_, exact) => (Some(exact.clone()), Some(exact)),
			AstRange::None(..) => (None, None),
		};

		if let (Some(min), Some(max)) = min_max.clone() {
			if min.value() > max.value() {
				self.report(Report::InvalidRange {
					span: ast_span,
					min_span: min.span(),
				});

				(None, None)
			} else {
				min_max
			}
		} else {
			min_max
		}
	}

	pub fn range_f32(&mut self, ast: AstRange) -> HirRange<f32> {
		let (min, max) = self.unwrap_astrange(ast);

		HirRange::new(min.map(|num| num.value() as f32), max.map(|num| num.value() as f32))
	}

	pub fn range_f64(&mut self, ast: AstRange) -> HirRange<f64> {
		let (min, max) = self.unwrap_astrange(ast);

		HirRange::new(min.map(|num| num.value()), max.map(|num| num.value()))
	}
}

macro_rules! impl_range {
	($n:ident, $T:ty) => {
		impl<'a> HirBuilder<'a> {
			pub fn $n(&mut self, ast: AstRange) -> HirRange<$T> {
				let (min, max) = self.unwrap_astrange(ast);

				HirRange::new(
					min.map(|num| {
						let value = num.value();

						if value.fract() != 0.0 {
							self.report(Report::ExpectedIntegerFoundNumber {
								span: num.span(),
								value,
							});
						}

						if (<$T>::MIN as f64) > value {
							self.report(Report::NumberBelowRange {
								span: num.span(),
								min: Box::new(<$T>::MIN),
							});
						} else if (<$T>::MAX as f64) < value {
							self.report(Report::NumberAboveRange {
								span: num.span(),
								max: Box::new(<$T>::MAX),
							});
						}

						value as $T
					}),
					max.map(|num| {
						let value = num.value();

						if value.fract() != 0.0 {
							self.report(Report::ExpectedIntegerFoundNumber {
								span: num.span(),
								value,
							});
						}

						if (<$T>::MIN as f64) > value {
							self.report(Report::NumberBelowRange {
								span: num.span(),
								min: Box::new(<$T>::MIN),
							});
						} else if (<$T>::MAX as f64) < value {
							self.report(Report::NumberAboveRange {
								span: num.span(),
								max: Box::new(<$T>::MAX),
							});
						}

						value as $T
					}),
				)
			}
		}
	};
}

impl_range!(range_u8, u8);
impl_range!(range_i8, i8);
impl_range!(range_u16, u16);
impl_range!(range_i16, i16);
impl_range!(range_u32, u32);
impl_range!(range_i32, i32);
