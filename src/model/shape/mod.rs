macro_rules! impl_parry2d {
    ($type:ident) => {
        use parry2d_f64::bounding_volume::{BoundingSphere, AABB};
        use parry2d_f64::mass_properties::MassProperties;
        use parry2d_f64::math::{Isometry, Point, Real, Vector};
        use parry2d_f64::query::{PointProjection, PointQuery, Ray, RayCast, RayIntersection};
        use parry2d_f64::shape::{
            FeatureId, PolygonalFeatureMap, Shape, SimdCompositeShape, SupportMap, TypedShape,
        };

        impl PointQuery for $type {
            fn project_local_point(&self, pt: &Point<Real>, solid: bool) -> PointProjection {
                self.as_parry().project_local_point(pt, solid)
            }

            fn project_local_point_and_get_feature(
                &self,
                pt: &Point<Real>,
            ) -> (PointProjection, FeatureId) {
                self.as_parry().project_local_point_and_get_feature(pt)
            }

            fn distance_to_local_point(&self, pt: &Point<Real>, solid: bool) -> Real {
                self.as_parry().distance_to_local_point(pt, solid)
            }

            fn contains_local_point(&self, pt: &Point<Real>) -> bool {
                self.as_parry().contains_local_point(pt)
            }

            fn project_point(
                &self,
                m: &Isometry<Real>,
                pt: &Point<Real>,
                solid: bool,
            ) -> PointProjection {
                self.as_parry().project_point(m, pt, solid)
            }

            fn distance_to_point(&self, m: &Isometry<Real>, pt: &Point<Real>, solid: bool) -> Real {
                self.as_parry().distance_to_point(m, pt, solid)
            }

            fn project_point_and_get_feature(
                &self,
                m: &Isometry<Real>,
                pt: &Point<Real>,
            ) -> (PointProjection, FeatureId) {
                self.as_parry().project_point_and_get_feature(m, pt)
            }

            fn contains_point(&self, m: &Isometry<Real>, pt: &Point<Real>) -> bool {
                self.as_parry().contains_point(m, pt)
            }
        }

        impl RayCast for $type {
            fn cast_local_ray(&self, ray: &Ray, max_toi: Real, solid: bool) -> Option<Real> {
                self.as_parry().cast_local_ray(ray, max_toi, solid)
            }

            fn cast_local_ray_and_get_normal(
                &self,
                ray: &Ray,
                max_toi: Real,
                solid: bool,
            ) -> Option<RayIntersection> {
                self.as_parry().cast_local_ray_and_get_normal(ray, max_toi, solid)
            }

            fn intersects_local_ray(&self, ray: &Ray, max_toi: Real) -> bool {
                self.as_parry().intersects_local_ray(ray, max_toi)
            }

            fn cast_ray(
                &self,
                m: &Isometry<Real>,
                ray: &Ray,
                max_toi: Real,
                solid: bool,
            ) -> Option<Real> {
                self.as_parry().cast_ray(m, ray, max_toi, solid)
            }

            fn cast_ray_and_get_normal(
                &self,
                m: &Isometry<Real>,
                ray: &Ray,
                max_toi: Real,
                solid: bool,
            ) -> Option<RayIntersection> {
                self.as_parry().cast_ray_and_get_normal(m, ray, max_toi, solid)
            }

            fn intersects_ray(&self, m: &Isometry<Real>, ray: &Ray, max_toi: Real) -> bool {
                self.as_parry().intersects_ray(m, ray, max_toi)
            }
        }

        impl Shape for $type {
            fn compute_local_aabb(&self) -> AABB {
                self.as_parry().compute_local_aabb()
            }

            fn compute_local_bounding_sphere(&self) -> BoundingSphere {
                self.as_parry().compute_local_bounding_sphere()
            }

            fn clone_box(&self) -> Box<dyn Shape> {
                self.as_parry().clone_box()
            }

            fn compute_aabb(&self, position: &Isometry<Real>) -> AABB {
                self.as_parry().compute_aabb(position)
            }

            fn compute_bounding_sphere(&self, position: &Isometry<Real>) -> BoundingSphere {
                self.as_parry().compute_bounding_sphere(position)
            }

            fn mass_properties(&self, density: Real) -> MassProperties {
                self.as_parry().mass_properties(density)
            }

            fn shape_type(&self) -> parry2d_f64::shape::ShapeType {
                self.as_parry().shape_type()
            }

            fn as_typed_shape(&self) -> TypedShape<'_> {
                self.as_parry().as_typed_shape()
            }

            fn ccd_thickness(&self) -> Real {
                self.as_parry().ccd_thickness()
            }

            fn ccd_angular_thickness(&self) -> Real {
                self.as_parry().ccd_angular_thickness()
            }

            fn is_convex(&self) -> bool {
                self.as_parry().is_convex()
            }

            fn as_support_map(&self) -> Option<&dyn SupportMap> {
                self.as_parry().as_support_map()
            }

            fn as_composite_shape(&self) -> Option<&dyn SimdCompositeShape> {
                self.as_parry().as_composite_shape()
            }

            fn as_polygonal_feature_map(&self) -> Option<(&dyn PolygonalFeatureMap, Real)> {
                self.as_parry().as_polygonal_feature_map()
            }

            fn feature_normal_at_point(
                &self,
                feature: FeatureId,
                point: &Point<Real>,
            ) -> Option<nalgebra::Unit<Vector<Real>>> {
                self.as_parry().feature_normal_at_point(feature, point)
            }

            fn compute_swept_aabb(
                &self,
                start_pos: &Isometry<Real>,
                end_pos: &Isometry<Real>,
            ) -> AABB {
                self.as_parry().compute_swept_aabb(start_pos, end_pos)
            }
        }
    };
}

pub mod arc;
pub mod circle;
pub mod path;
pub mod polygon;
pub mod rt;
pub mod shape_type;
