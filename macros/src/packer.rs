extern crate usbd_hid_descriptors;

use crate::spec::*;

pub fn uses_report_ids(spec: &Spec) -> bool {
    match spec {
        Spec::MainItem(_) => false,
        Spec::Collection(c) => {
            for (_, s) in &c.fields {
                if uses_report_ids(&s) {
                    return true;
                }
            }
            c.report_id.is_some()
        }
    }
}
