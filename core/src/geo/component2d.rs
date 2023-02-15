use {
    crate::{
        geo::geo_utils::orient,
        index::point_values::Relation,
        util::float::{f64_max, f64_min},
    },
};

/**
 * Used by withinTriangle to check the within relationship between a triangle and the query shape
 * (e.g. if the query shape is within the triangle).
 */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WithinRelation {
    /**
     * If the shape is a candidate for within. Typically this is return if the query shape is fully
     * inside the triangle or if the query shape intersects only edges that do not belong to the
     * original shape.
     */
    Candidate,

    /**
     * The query shape intersects an edge that does belong to the original shape or any point of the
     * triangle is inside the shape.
     */
    NotWithin,

    /** The query shape is disjoint with the triangle. */
    Disjoint,
}

/**
 * 2D Geometry object that supports spatial relationships with bounding boxes, triangles and points.
 *
 * @lucene.internal
 */
pub trait Component2D {
    /** min X value for the component * */
    fn getMinX() -> f64;

    /** max X value for the component * */
    fn getMaxX() -> f64;

    /** min Y value for the component * */
    fn getMinY() -> f64;

    /** max Y value for the component * */
    fn getMaxY() -> f64;

    /** relates this component2D with a point * */
    fn contains(x: f64, y: f64) -> bool;

    /** relates this component2D with a bounding box * */
    fn relate(minX: f64, maxX: f64, minY: f64, maxY: f64) -> Relation;

    /** return true if this component2D intersects the provided line * */
    fn intersectsLineMinMax(minX: f64, maxX: f64, minY: f64, maxY: f64, aX: f64, aY: f64, bX: f64, bY: f64) -> bool;

    /** return true if this component2D intersects the provided triangle * */
    fn intersectsTriangleMinMax(
        minX: f64,
        maxX: f64,
        minY: f64,
        maxY: f64,
        aX: f64,
        aY: f64,
        bX: f64,
        bY: f64,
        cX: f64,
        cY: f64,
    ) -> bool;

    /** return true if this component2D contains the provided line * */
    fn containsLineMinMax(minX: f64, maxX: f64, minY: f64, maxY: f64, aX: f64, aY: f64, bX: f64, bY: f64) -> bool;

    /** return true if this component2D contains the provided triangle * */
    fn containsTriangleMinMax(
        minX: f64,
        maxX: f64,
        minY: f64,
        maxY: f64,
        aX: f64,
        aY: f64,
        bX: f64,
        bY: f64,
        cX: f64,
        cY: f64,
    ) -> bool;

    /** Compute the within relation of this component2D with a point * */
    fn withinPoint(x: f64, y: f64) -> WithinRelation;

    /** Compute the within relation of this component2D with a line * */
    fn withinLineMinMax(
        minX: f64,
        maxX: f64,
        minY: f64,
        maxY: f64,
        aX: f64,
        aY: f64,
        ab: bool,
        bX: f64,
        bY: f64,
    ) -> WithinRelation;

    /** Compute the within relation of this component2D with a triangle * */
    fn withinTriangleMinMax(
        minX: f64,
        maxX: f64,
        minY: f64,
        maxY: f64,
        aX: f64,
        aY: f64,
        ab: bool,
        bX: f64,
        bY: f64,
        bc: bool,
        cX: f64,
        cY: f64,
        ca: bool,
    ) -> WithinRelation;

    /** return true if this component2D intersects the provided line * */
    fn intersectsLine(aX: f64, aY: f64, bX: f64, bY: f64) -> bool {
        let minY = f64_min(aY, bY);
        let minX = f64_min(aX, bX);
        let maxY = f64_max(aY, bY);
        let maxX = f64_max(aX, bX);
        Self::intersectsLineMinMax(minX, maxX, minY, maxY, aX, aY, bX, bY)
    }

    /** return true if this component2D intersects the provided triangle * */
    fn intersectsTriangle(aX: f64, aY: f64, bX: f64, bY: f64, cX: f64, cY: f64) -> bool {
        let minY = f64_min(f64_min(aY, bY), cY);
        let minX = f64_min(f64_min(aX, bX), cX);
        let maxY = f64_max(f64_max(aY, bY), cY);
        let maxX = f64_max(f64_max(aX, bX), cX);
        Self::intersectsTriangleMinMax(minX, maxX, minY, maxY, aX, aY, bX, bY, cX, cY)
    }

    /** return true if this component2D contains the provided line * */
    fn containsLine(aX: f64, aY: f64, bX: f64, bY: f64) -> bool {
        let minY = f64_min(aY, bY);
        let minX = f64_min(aX, bX);
        let maxY = f64_max(aY, bY);
        let maxX = f64_max(aX, bX);
        Self::containsLineMinMax(minX, maxX, minY, maxY, aX, aY, bX, bY)
    }

    /** return true if this component2D contains the provided triangle * */
    fn containsTriangle(aX: f64, aY: f64, bX: f64, bY: f64, cX: f64, cY: f64) -> bool {
        let minY = f64_min(f64_min(aY, bY), cY);
        let minX = f64_min(f64_min(aX, bX), cX);
        let maxY = f64_max(f64_max(aY, bY), cY);
        let maxX = f64_max(f64_max(aX, bX), cX);
        Self::containsTriangleMinMax(minX, maxX, minY, maxY, aX, aY, bX, bY, cX, cY)
    }

    /** Compute the within relation of this component2D with a triangle * */
    fn withinLine(aX: f64, aY: f64, ab: bool, bX: f64, bY: f64) -> WithinRelation {
        let minY = f64_min(aY, bY);
        let minX = f64_min(aX, bX);
        let maxY = f64_max(aY, bY);
        let maxX = f64_max(aX, bX);
        Self::withinLineMinMax(minX, maxX, minY, maxY, aX, aY, ab, bX, bY)
    }

    /** Compute the within relation of this component2D with a triangle * */
    fn withinTriangle(
        aX: f64,
        aY: f64,
        ab: bool,
        bX: f64,
        bY: f64,
        bc: bool,
        cX: f64,
        cY: f64,
        ca: bool,
    ) -> WithinRelation {
        let minY = f64_min(f64_min(aY, bY), cY);
        let minX = f64_min(f64_min(aX, bX), cX);
        let maxY = f64_max(f64_max(aY, bY), cY);
        let maxX = f64_max(f64_max(aX, bX), cX);
        Self::withinTriangleMinMax(minX, maxX, minY, maxY, aX, aY, ab, bX, bY, bc, cX, cY, ca)
    }

    /** Compute whether the bounding boxes are disjoint * */
    fn disjoint(
        minX1: f64,
        maxX1: f64,
        minY1: f64,
        maxY1: f64,
        minX2: f64,
        maxX2: f64,
        minY2: f64,
        maxY2: f64,
    ) -> bool {
        maxY1 < minY2 || minY1 > maxY2 || maxX1 < minX2 || minX1 > maxX2
    }

    /** Compute whether the first bounding box 1 is within the second bounding box * */
    fn within(minX1: f64, maxX1: f64, minY1: f64, maxY1: f64, minX2: f64, maxX2: f64, minY2: f64, maxY2: f64) -> bool {
        minY2 <= minY1 && maxY2 >= maxY1 && minX2 <= minX1 && maxX2 >= maxX1
    }

    /** returns true if rectangle (defined by minX, maxX, minY, maxY) contains the X Y point */
    fn containsPoint(x: f64, y: f64, minX: f64, maxX: f64, minY: f64, maxY: f64) -> bool {
        x >= minX && x <= maxX && y >= minY && y <= maxY
    }

    /** Compute whether the given x, y point is in a triangle; uses the winding order method */
    fn pointInTriangle(
        minX: f64,
        maxX: f64,
        minY: f64,
        maxY: f64,
        x: f64,
        y: f64,
        aX: f64,
        aY: f64,
        bX: f64,
        bY: f64,
        cX: f64,
        cY: f64,
    ) -> bool {
        // check the bounding box because if the triangle is degenerated, e.g points and lines, we need
        // to filter out
        // coplanar points that are not part of the triangle.
        if x >= minX && x <= maxX && y >= minY && y <= maxY {
            let a = orient(x, y, aX, aY, bX, bY);
            let b = orient(x, y, bX, bY, cX, cY);

            if a == 0 || b == 0 || (a < 0) == (b < 0) {
                let c = orient(x, y, cX, cY, aX, aY);
                c == 0 || ((c < 0) == (b < 0 || a < 0))
            } else {
                false
            }
        } else {
            false
        }
    }
}
