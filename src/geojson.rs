use serde::de::*;
use serde::ser::*;
use serde::*;
use std::fmt;
use std::marker::PhantomData;

use crate::types::*;

const WGS84_SRID: Option<u32> = Some(4326);

macro_rules! check_srid_wgs84 {
    ($x:ident) => {
        if $x.srid.is_some() && $x.srid != WGS84_SRID {
            return Err(ser::Error::custom(format!(
                "Invalid SRID {}",
                $x.srid.unwrap()
            )));
        }
    };
}

pub trait GeoJsonGeometry<V> {
    fn to_geo_coordinates(&self) -> Vec<V>;
    fn from_geo_coordinates(coordinates: Vec<V>) -> Result<Self, PointConstructorError>
    where
        Self: Sized;
}

struct GeometryVisitor<T, V> {
    expected_type: String,
    marker: PhantomData<fn() -> (T, V)>,
}

impl<T, V> GeometryVisitor<T, V> {
    fn new(expected_type: &str) -> Self {
        GeometryVisitor {
            expected_type: expected_type.to_string(),
            marker: PhantomData,
        }
    }
}

impl<'de, T: GeoJsonGeometry<V>, V: Deserialize<'de>> Visitor<'de> for GeometryVisitor<T, V> {
    // The type that our Visitor is going to produce.
    type Value = T;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("a GeoJson {}", self.expected_type))
    }

    // Deserialize geometry from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
        V: Deserialize<'de>,
    {
        let mut point: Option<T> = None;
        while let Some(key) = access.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    let t = access.next_value::<String>()?;
                    if t != self.expected_type {
                        return Err(de::Error::custom(format_args!(
                            "unknown type `{}`, expected {}",
                            t, self.expected_type
                        )));
                    }
                }
                "coordinates" => {
                    let v = access.next_value::<Vec<V>>()?;
                    match T::from_geo_coordinates(v) {
                        Ok(p) => point = Some(p),
                        Err(err) => {
                            return Err(de::Error::custom(format!("invalid coordinates: {}", err)));
                        }
                    }
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["type", "coordinates"]));
                }
            }
        }

        if point.is_none() {
            return Err(de::Error::missing_field("coordinates"));
        }

        Ok(point.unwrap())
    }
}

impl GeoJsonGeometry<f64> for Point {
    fn to_geo_coordinates(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }

    fn from_geo_coordinates(coordinates: Vec<f64>) -> Result<Self, PointConstructorError> {
        if coordinates.len() != 2 {
            return Err(PointConstructorError {
                reason: format!("invalid size {:?} for Point", coordinates.len()).to_string(),
            });
        }
        Ok(Point::new(coordinates[0], coordinates[1], WGS84_SRID))
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("type", "Point")?;
        state.serialize_field("coordinates", &self.to_geo_coordinates())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<Point, f64>::new("Point"))
    }
}

impl GeoJsonGeometry<f64> for PointZ {
    fn to_geo_coordinates(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }

    fn from_geo_coordinates(coordinates: Vec<f64>) -> Result<Self, PointConstructorError> {
        if coordinates.len() != 3 {
            return Err(PointConstructorError {
                reason: format!("invalid size {:?} for PointZ", coordinates.len()).to_string(),
            });
        }
        Ok(PointZ::new(
            coordinates[0],
            coordinates[1],
            coordinates[2],
            WGS84_SRID,
        ))
    }
}

impl Serialize for PointZ {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("type", "Point")?;
        state.serialize_field("coordinates", &self.to_geo_coordinates())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PointZ {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<PointZ, f64>::new("Point"))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> GeoJsonGeometry<Vec<f64>> for MultiPoint<T> {
    fn to_geo_coordinates(&self) -> Vec<Vec<f64>> {
        self.points
            .iter()
            .map(|p| p.to_geo_coordinates())
            .collect::<Vec<Vec<f64>>>()
    }

    fn from_geo_coordinates(coordinates: Vec<Vec<f64>>) -> Result<Self, PointConstructorError> {
        let mut multi_point = MultiPoint::<T> {
            points: vec![],
            srid: WGS84_SRID,
        };
        for p in coordinates {
            multi_point.points.push(T::from_geo_coordinates(p)?);
        }
        Ok(multi_point)
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> Serialize for MultiPoint<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let coordinates = self.to_geo_coordinates();

        let mut state = serializer.serialize_struct("MultiPoint", 2)?;
        state.serialize_field("type", "MultiPoint")?;
        state.serialize_field("coordinates", &coordinates)?;
        state.end()
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT> Deserialize<'de> for MultiPoint<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<MultiPoint<T>, Vec<f64>>::new(
            "MultiPoint",
        ))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> GeoJsonGeometry<Vec<f64>> for LineString<T> {
    fn to_geo_coordinates(&self) -> Vec<Vec<f64>> {
        self.points
            .iter()
            .map(|p| p.to_geo_coordinates())
            .collect::<Vec<Vec<f64>>>()
    }

    fn from_geo_coordinates(coordinates: Vec<Vec<f64>>) -> Result<Self, PointConstructorError> {
        let mut multi_point = LineString::<T> {
            points: vec![],
            srid: WGS84_SRID,
        };
        for p in coordinates {
            multi_point.points.push(T::from_geo_coordinates(p)?);
        }
        Ok(multi_point)
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> Serialize for LineString<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let coordinates = self.to_geo_coordinates();

        let mut state = serializer.serialize_struct("LineString", 2)?;
        state.serialize_field("type", "LineString")?;
        state.serialize_field("coordinates", &coordinates)?;
        state.end()
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT> Deserialize<'de> for LineString<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<LineString<T>, Vec<f64>>::new(
            "LineString",
        ))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> GeoJsonGeometry<Vec<Vec<f64>>> for MultiLineString<T> {
    fn to_geo_coordinates(&self) -> Vec<Vec<Vec<f64>>> {
        self.lines
            .iter()
            .map(|line| line.to_geo_coordinates())
            .collect::<Vec<Vec<Vec<f64>>>>()
    }

    fn from_geo_coordinates(
        coordinates: Vec<Vec<Vec<f64>>>,
    ) -> Result<Self, PointConstructorError> {
        let mut multi_line_string = MultiLineString::<T> {
            lines: vec![],
            srid: WGS84_SRID,
        };
        for line in coordinates {
            multi_line_string
                .lines
                .push(LineString::from_geo_coordinates(line)?);
        }
        Ok(multi_line_string)
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> Serialize for MultiLineString<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let coordinates = self.to_geo_coordinates();

        let mut state = serializer.serialize_struct("MultiLineString", 2)?;
        state.serialize_field("type", "MultiLineString")?;
        state.serialize_field("coordinates", &coordinates)?;
        state.end()
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT> Deserialize<'de> for MultiLineString<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<MultiLineString<T>, Vec<Vec<f64>>>::new(
            "MultiLineString",
        ))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> GeoJsonGeometry<Vec<Vec<f64>>> for Polygon<T> {
    fn to_geo_coordinates(&self) -> Vec<Vec<Vec<f64>>> {
        self.rings
            .iter()
            .map(|ring| ring.iter().map(|p| p.to_geo_coordinates()).collect())
            .collect::<Vec<Vec<Vec<f64>>>>()
    }

    fn from_geo_coordinates(
        coordinates: Vec<Vec<Vec<f64>>>,
    ) -> Result<Self, PointConstructorError> {
        let mut polygon = Polygon::<T> {
            rings: vec![],
            srid: WGS84_SRID,
        };
        for ring in coordinates {
            polygon.add_ring();
            for p in ring {
                polygon.add_point(T::from_geo_coordinates(p)?);
            }
        }
        Ok(polygon)
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> Serialize for Polygon<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let coordinates = self.to_geo_coordinates();

        let mut state = serializer.serialize_struct("Polygon", 2)?;
        state.serialize_field("type", "Polygon")?;
        state.serialize_field("coordinates", &coordinates)?;
        state.end()
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT> Deserialize<'de> for Polygon<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<Polygon<T>, Vec<Vec<f64>>>::new("Polygon"))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> GeoJsonGeometry<Vec<Vec<Vec<f64>>>>
    for MultiPolygon<T>
{
    fn to_geo_coordinates(&self) -> Vec<Vec<Vec<Vec<f64>>>> {
        self.polygons
            .iter()
            .map(|polygon| polygon.to_geo_coordinates())
            .collect::<Vec<Vec<Vec<Vec<f64>>>>>()
    }

    fn from_geo_coordinates(
        coordinates: Vec<Vec<Vec<Vec<f64>>>>,
    ) -> Result<Self, PointConstructorError> {
        let mut multi_polygon = MultiPolygon::<T> {
            polygons: vec![],
            srid: WGS84_SRID,
        };
        for coordinate in coordinates {
            let polygon = Polygon::<T>::from_geo_coordinates(coordinate)?;
            multi_polygon.polygons.push(polygon);
        }
        Ok(multi_polygon)
    }
}

impl<T: GeoJsonGeometry<f64> + PointT> Serialize for MultiPolygon<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        check_srid_wgs84!(self);
        let coordinates = self.to_geo_coordinates();

        let mut state = serializer.serialize_struct("MultiPolygon", 2)?;
        state.serialize_field("type", "MultiPolygon")?;
        state.serialize_field("coordinates", &coordinates)?;
        state.end()
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT> Deserialize<'de> for MultiPolygon<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor::<MultiPolygon<T>, Vec<Vec<Vec<f64>>>>::new(
            "MultiPolygon",
        ))
    }
}

impl<T: GeoJsonGeometry<f64> + PointT + Serialize> Serialize for GeometryContainer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GeometryContainer::Point(g) => g.serialize(serializer),
            GeometryContainer::MultiPoint(g) => g.serialize(serializer),
            GeometryContainer::LineString(g) => g.serialize(serializer),
            GeometryContainer::MultiLineString(g) => g.serialize(serializer),
            GeometryContainer::Polygon(g) => g.serialize(serializer),
            GeometryContainer::MultiPolygon(g) => g.serialize(serializer),
            GeometryContainer::GeometryCollection(g) => g.serialize(serializer),
        }
    }
}

impl<T: GeoJsonGeometry<f64> + PointT + Serialize> Serialize for GeometryCollection<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GeometryCollection", 2)?;
        state.serialize_field("type", "GeometryCollection")?;
        state.serialize_field("geometries", &self.geometries)?;
        state.end()
    }
}

struct GeometryCollectionVisitor<T> {
    marker: PhantomData<fn() -> T>,
}

impl<T> GeometryCollectionVisitor<T> {
    fn new() -> Self {
        GeometryCollectionVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT + Deserialize<'de>> Visitor<'de>
    for GeometryCollectionVisitor<T>
{
    // The type that our Visitor is going to produce.
    type Value = GeometryCollection<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("a GeoJson GeometryCollection"))
    }

    // Deserialize GeometryCollection from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut geometries: Option<Vec<GeometryContainer<T>>> = None;
        while let Some(key) = access.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    let t = access.next_value::<String>()?;
                    if t != "GeometryCollection" {
                        return Err(de::Error::custom(format_args!(
                            "unknown type `{}`, expected GeometryCollection",
                            t
                        )));
                    }
                }
                "geometries" => {
                    let v = access.next_value::<Vec<GeometryContainer<T>>>()?;
                    geometries = Some(v);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["type", "coordinates"]));
                }
            }
        }

        if geometries.is_none() {
            return Err(de::Error::missing_field("geometries"));
        }

        Ok(GeometryCollection::<T> {
            geometries: geometries.unwrap(),
            srid: WGS84_SRID,
        })
    }
}

impl<'de, T: GeoJsonGeometry<f64> + PointT + Deserialize<'de>> Deserialize<'de>
    for GeometryCollection<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryCollectionVisitor::<T>::new())
    }
}

impl<T: GeoJsonGeometry<f64> + PointT + Serialize, P: Serialize> Serialize
    for Feature<T, P>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Feature", 2)?;
        state.serialize_field("type", "Feature")?;
        if let Some(id) = &self.id {
            state.serialize_field("id", &id)?;
        }
        state.serialize_field("geometry", &self.geometry)?;
        state.serialize_field("properties", &self.properties)?;
        state.end()
    }
}

impl<T: GeoJsonGeometry<f64> + PointT + Serialize, P: Serialize> Serialize
    for FeatureCollection<T, P>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FeatureCollection", 2)?;
        state.serialize_field("type", "FeatureCollection")?;
        state.serialize_field("features", &self.features)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestFeatureProperties {
        pub name: String,
        pub size: i32,
    }

    #[test]
    fn test_point_serde() {
        let point = Point::new(72.0, 64.0, WGS84_SRID);
        let expected_json = "{\"type\":\"Point\",\"coordinates\":[72.0,64.0]}";
        let point_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(point, point_from_json);
        let point_json = serde_json::to_string(&point).unwrap();
        assert_eq!(expected_json, point_json);
    }

    #[test]
    fn test_pointz_serde() {
        let point = PointZ::new(72.0, 64.0, 52.0, WGS84_SRID);
        let expected_json = "{\"type\":\"Point\",\"coordinates\":[72.0,64.0,52.0]}";
        let point_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(point, point_from_json);
        let point_json = serde_json::to_string(&point).unwrap();
        assert_eq!(expected_json, point_json);
    }

    #[test]
    fn test_multi_point_serde() {
        let mut multi_point = MultiPoint::<Point> {
            points: vec![],
            srid: WGS84_SRID,
        };
        multi_point.add_point(Point::new(1.0, 2.0, WGS84_SRID));
        multi_point.add_point(Point::new(3.0, 4.0, WGS84_SRID));
        multi_point.add_point(Point::new(5.0, 6.0, WGS84_SRID));

        let expected_json =
            "{\"type\":\"MultiPoint\",\"coordinates\":[[1.0,2.0],[3.0,4.0],[5.0,6.0]]}";
        let multi_point_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(multi_point, multi_point_from_json);
        let multi_point_json = serde_json::to_string(&multi_point).unwrap();
        assert_eq!(expected_json, multi_point_json);
    }

    #[test]
    fn test_line_string_serde() {
        let mut line_string = LineString::<Point> {
            points: vec![],
            srid: WGS84_SRID,
        };
        line_string.add_point(Point::new(1.0, 2.0, WGS84_SRID));
        line_string.add_point(Point::new(3.0, 4.0, WGS84_SRID));
        line_string.add_point(Point::new(5.0, 6.0, WGS84_SRID));

        let expected_json =
            "{\"type\":\"LineString\",\"coordinates\":[[1.0,2.0],[3.0,4.0],[5.0,6.0]]}";
        let line_string_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(line_string, line_string_from_json);
        let line_string_json = serde_json::to_string(&line_string).unwrap();
        assert_eq!(expected_json, line_string_json);
    }

    #[test]
    fn test_multi_line_string_serde() {
        let mut multi_line_string = MultiLineString::<Point> {
            lines: vec![],
            srid: WGS84_SRID,
        };
        multi_line_string.add_point(Point::new(1.0, 2.0, WGS84_SRID));
        multi_line_string.add_point(Point::new(3.0, 4.0, WGS84_SRID));
        multi_line_string.add_point(Point::new(5.0, 6.0, WGS84_SRID));

        let expected_json =
            "{\"type\":\"MultiLineString\",\"coordinates\":[[[1.0,2.0],[3.0,4.0],[5.0,6.0]]]}";
        let multi_line_string_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(multi_line_string, multi_line_string_from_json);
        let multi_line_string_json = serde_json::to_string(&multi_line_string).unwrap();
        assert_eq!(expected_json, multi_line_string_json);
    }

    #[test]
    fn test_polygon_serde() {
        let mut polygon = Polygon::<Point> {
            rings: vec![],
            srid: WGS84_SRID,
        };
        polygon.add_point(Point::new(1.0, 2.0, WGS84_SRID));
        polygon.add_point(Point::new(3.0, 4.0, WGS84_SRID));
        polygon.add_point(Point::new(5.0, 6.0, WGS84_SRID));
        polygon.add_point(Point::new(1.0, 2.0, WGS84_SRID));

        let expected_json =
            "{\"type\":\"Polygon\",\"coordinates\":[[[1.0,2.0],[3.0,4.0],[5.0,6.0],[1.0,2.0]]]}";
        let polygon_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(polygon, polygon_from_json);
        let polygon_json = serde_json::to_string(&polygon).unwrap();
        assert_eq!(expected_json, polygon_json);
    }

    #[test]
    fn test_multi_polygon_serde() {
        let mut polygon = Polygon::<PointZ> {
            rings: vec![],
            srid: WGS84_SRID,
        };
        polygon.add_point(PointZ::new(1.0, 2.0, 3.0, WGS84_SRID));
        polygon.add_point(PointZ::new(4.0, 5.0, 6.0, WGS84_SRID));
        polygon.add_point(PointZ::new(7.0, 8.0, 9.0, WGS84_SRID));
        polygon.add_point(PointZ::new(1.0, 2.0, 3.0, WGS84_SRID));

        let multi_polygon = MultiPolygon::<PointZ> {
            polygons: vec![polygon],
            srid: WGS84_SRID,
        };

        let expected_json =
            "{\"type\":\"MultiPolygon\",\"coordinates\":[[[[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0],[1.0,2.0,3.0]]]]}";
        let multi_polygon_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(multi_polygon, multi_polygon_from_json);
        let multi_polygon_json = serde_json::to_string(&multi_polygon).unwrap();
        assert_eq!(expected_json, multi_polygon_json);
    }

    #[test]
    fn test_geometry_collection_serde() {
        let point = Point::new(1.0, 2.0, WGS84_SRID);
        let mut line_string = LineString::<Point> {
            points: vec![],
            srid: WGS84_SRID,
        };
        line_string.add_point(Point::new(3.0, 4.0, WGS84_SRID));
        line_string.add_point(Point::new(5.0, 6.0, WGS84_SRID));

        let geometry_collection = GeometryCollection::<Point> {
            geometries: vec![
                GeometryContainer::Point(point),
                GeometryContainer::LineString(line_string),
            ],
            srid: WGS84_SRID,
        };

        let expected_json =
            "{\"type\":\"GeometryCollection\",\"geometries\":[{\"type\":\"Point\",\"coordinates\":[1.0,2.0]},{\"type\":\"LineString\",\"coordinates\":[[3.0,4.0],[5.0,6.0]]}]}";
        let geometry_collection_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(geometry_collection, geometry_collection_from_json);
        let geometry_collection_json = serde_json::to_string(&geometry_collection).unwrap();
        assert_eq!(expected_json, geometry_collection_json);
    }

    #[test]
    fn test_feature_collection_serde() {
        let point = Point::new(1.0, 2.0, WGS84_SRID);

        let feature1 = Feature::<Point, TestFeatureProperties> {
            id: None,
            geometry: Some(GeometryContainer::Point(point)),
            properties: Some(TestFeatureProperties {
                name: "Test".to_string(),
                size: 123,
            }),
        };

        let feature2 = Feature::<Point, _> {
            id: Some("Test".to_string()),
            geometry: None,
            properties: None,
        };

        let feature_collection = FeatureCollection::<Point, _> {
            features: vec![feature1, feature2],
        };

        let expected_json =
            "{\"type\":\"FeatureCollection\",\"features\":[{\"type\":\"Feature\",\"geometry\":{\"type\":\"Point\",\"coordinates\":[1.0,2.0]},\"properties\":{\"name\":\"Test\",\"size\":123}},{\"type\":\"Feature\",\"id\":\"Test\",\"geometry\":null,\"properties\":null}]}";
        let feature_collection_from_json = serde_json::from_str(expected_json).unwrap();
        assert_eq!(feature_collection, feature_collection_from_json);
        let feature_collection_json = serde_json::to_string(&feature_collection).unwrap();
        assert_eq!(expected_json, feature_collection_json);
    }

    #[test]
    fn test_non_wgs84_serde() {
        let point = Point::new(72.0, 64.0, Some(4324));
        let point_json = serde_json::to_string(&point);
        assert_eq!(true, point_json.is_err());
    }

    #[test]
    fn test_no_srid_serde() {
        let point = Point::new(72.0, 64.0, None);
        let point_json = serde_json::to_string(&point);
        assert_eq!(true, point_json.is_ok());
    }
}
