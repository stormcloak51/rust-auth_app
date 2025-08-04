use actix_web::guard::{Guard, GuardContext};

use crate::models::auth::{Claims, UserRole};

pub struct RoleGuard {
	pub required_role: UserRole,
}

impl Guard for RoleGuard {
    fn check(&self, ctx: &GuardContext) -> bool {
        let exts = ctx.req_data();

        match exts.get::<Claims>() {
            Some(value) => {
							return value.role == self.required_role;
						},
            None => false,
        }
    }
}
