use crate::model::primitive::rt::Rt;
use crate::model::pt::Pt;

pub fn point_cloud_bounds(pts: &[Pt]) -> Rt {
    if pts.is_empty() {
        Rt::default()
    } else {
        let mut bl = pts[0];
        let mut tr = pts[0];
        for pt in pts.iter() {
            bl.x = bl.x.min(pt.x);
            bl.y = bl.y.min(pt.y);
            tr.x = tr.x.max(pt.x);
            tr.y = tr.y.max(pt.y);
        }
        Rt::enclosing(bl, tr)
    }
}
