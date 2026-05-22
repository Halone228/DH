use std::sync::Arc;

use dayhelper_application::{AcceptDesktopSync, RedeemPairCode};
use dayhelper_ports::{DesktopTokenRepo, UserRepo};

#[derive(Clone)]
pub struct ServerDesktopState {
    pub redeem_pair_code: Arc<RedeemPairCode>,
    pub accept_sync: Arc<AcceptDesktopSync>,
    pub tokens: Arc<dyn DesktopTokenRepo>,
    pub users: Arc<dyn UserRepo>,
}
