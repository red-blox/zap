use super::ty::HirTy;

#[derive(Debug, Clone, Copy)]
pub struct HirTyDeclId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct HirRemoteId(pub usize);

#[derive(Debug, Clone, Copy)]
pub enum HirRemoteBatching {
	None,
	MaxTime(f64),
}

#[derive(Debug, Clone)]
pub struct HirRemote {
	reliable: bool,
	batching: HirRemoteBatching,
}

impl HirRemote {
	pub fn new(reliable: bool, batching: HirRemoteBatching) -> Self {
		Self { reliable, batching }
	}
}

#[derive(Debug, Clone, Copy)]
pub enum HirEventSource {
	Server,
	Client,
}

#[derive(Debug, Clone)]
pub struct HirEvent {
	from: HirEventSource,
	over: HirRemoteId,
	tys: Vec<HirTy>,
}

impl HirEvent {
	pub fn new(from: HirEventSource, over: HirRemoteId) -> Self {
		Self {
			from,
			over,
			tys: Vec::new(),
		}
	}

	pub fn add_tys(mut self, tys: Vec<HirTy>) -> Self {
		self.tys = tys;
		self
	}
}
