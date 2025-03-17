use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    rc::Rc,
};

use crate::{
    Attribute, AttributeData, Geometry, Location, SubblockMode, Vector3, array,
    colormap::NumberRange,
};

use super::{Problems, Reason};

pub trait Validate {
    #[doc(hidden)]
    fn validate_inner(&mut self, _val: &mut Validator);

    /// Call to validate the object, returning errors and warnings.
    ///
    /// The `Ok` value is a `Problems` object contain that is either empty or contains
    /// only warnings. The `Err` value is a `Problems` object containing at least one
    /// error.
    fn validate(&mut self) -> Result<Problems, Problems> {
        let mut val = Validator::new();
        self.validate_inner(&mut val);
        val.finish().into_result()
    }
}

impl<T: Validate> Validate for Option<T> {
    #[doc(hidden)]
    fn validate_inner(&mut self, val: &mut Validator) {
        if let Some(obj) = self {
            obj.validate_inner(val);
        }
    }
}

fn normalise([x, y, z]: Vector3) -> Vector3 {
    let mag = (x * x + y * y + z * z).sqrt();
    if mag == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [x / mag, y / mag, z / mag]
    }
}

fn ortho(a: Vector3, b: Vector3) -> bool {
    const THRESHOLD: f64 = 1e-6;
    let [x0, y0, z0] = normalise(a);
    let [x1, y1, z1] = normalise(b);
    (x0 * x1 + y0 * y1 + z0 * z1).abs() < THRESHOLD
}

#[derive(Debug)]
pub struct Validator<'n> {
    filenames: Rc<Option<HashSet<String>>>,
    problems: Rc<RefCell<Problems>>,
    ty: &'static str,
    name: Option<&'n str>,
    limit: Option<usize>,
    extra_errors: u32,
    extra_warnings: u32,
}

impl<'n> Validator<'n> {
    pub(crate) fn new() -> Self {
        Self {
            filenames: Default::default(),
            problems: Default::default(),
            ty: "",
            name: None,
            limit: None,
            extra_errors: 0,
            extra_warnings: 0,
        }
    }

    pub(crate) fn finish(mut self) -> Problems {
        if self.extra_warnings > 0 {
            self.push(Reason::MoreWarnings(self.extra_warnings), None);
        }
        if self.extra_errors > 0 {
            self.push(Reason::MoreErrors(self.extra_errors), None);
        }
        self.problems.take()
    }

    fn push_full(
        &mut self,
        reason: Reason,
        ty: &'static str,
        field: Option<&'static str>,
        name: Option<&str>,
    ) {
        let mut problems = self.problems.borrow_mut();
        match self.limit {
            Some(limit) if problems.len() >= limit => {
                if reason.is_error() {
                    self.extra_errors += 1;
                } else {
                    self.extra_warnings += 1;
                }
            }
            _ => {
                problems.push(reason, ty, field, name.map(ToOwned::to_owned));
            }
        }
    }

    fn push(&mut self, reason: Reason, field: Option<&'static str>) {
        self.push_full(reason, self.ty, field, self.name);
    }

    pub(crate) fn with_filenames<I, T>(mut self, filenames: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.filenames = Some(filenames.into_iter().map(Into::into).collect()).into();
        self
    }

    pub(crate) fn with_limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit.map(|n| n.try_into().expect("u32 fits in usize"));
        self
    }

    pub(crate) fn enter(&mut self, ty: &'static str) -> Self {
        Validator {
            filenames: self.filenames.clone(),
            problems: self.problems.clone(),
            ty,
            name: self.name,
            limit: None,
            extra_errors: 0,
            extra_warnings: 0,
        }
    }

    pub(crate) fn name(mut self, name: &'n str) -> Self {
        self.name = Some(name);
        self
    }

    pub(crate) fn obj(mut self, obj: &mut impl Validate) -> Self {
        obj.validate_inner(&mut self);
        self
    }

    pub(crate) fn array(
        mut self,
        array: &mut array::Array<impl array::ArrayType>,
        constraint: array::Constraint,
        field: &'static str,
    ) -> Self {
        _ = array
            .constrain(constraint)
            .map_err(|reason| self.push(reason, Some(field)));
        for reason in array.run_write_checks() {
            self.push(reason, Some(field));
        }
        if let Some(filenames) = self.filenames.as_ref() {
            if !filenames.contains(array.filename()) {
                self.push(
                    Reason::ZipMemberMissing(array.filename().to_owned()),
                    Some(field),
                );
            }
        }
        self
    }

    pub(crate) fn array_opt(
        self,
        array_opt: Option<&mut array::Array<impl array::ArrayType>>,
        constraint: array::Constraint,
        field: &'static str,
    ) -> Self {
        if let Some(array) = array_opt {
            self.array(array, constraint, field)
        } else {
            self
        }
    }

    pub(crate) fn objs<'c>(
        mut self,
        objs: impl IntoIterator<Item = &'c mut (impl Validate + 'c)>,
    ) -> Self {
        for obj in objs {
            self = self.obj(obj);
        }
        self
    }

    pub(crate) fn grid_count(mut self, count: &[u64]) -> Self {
        const MAX: u64 = u32::MAX as u64;
        if count.iter().any(|n| *n > MAX) {
            self.push(Reason::GridTooLarge(count.to_vec()), None);
        }
        self
    }

    pub(crate) fn subblock_mode_and_count(
        mut self,
        mode: Option<SubblockMode>,
        count: [u32; 3],
    ) -> Self {
        if mode == Some(SubblockMode::Octree) && !count.iter().all(|n| n.is_power_of_two()) {
            self.push(Reason::OctreeNotPowerOfTwo(count), None);
        }
        self
    }

    pub(crate) fn finite(mut self, value: f64, field: &'static str) -> Self {
        if !value.is_finite() {
            self.push(Reason::NotFinite, Some(field));
        }
        self
    }

    pub(crate) fn finite_seq(
        mut self,
        values: impl IntoIterator<Item = f64>,
        field: &'static str,
    ) -> Self {
        for value in values {
            if !value.is_finite() {
                self.push(Reason::NotFinite, Some(field));
                break;
            }
        }
        self
    }

    pub(crate) fn above_zero<T>(mut self, value: T, field: &'static str) -> Self
    where
        T: Default + PartialOrd,
    {
        if value <= T::default() {
            self.push(Reason::NotGreaterThanZero, Some(field));
        }
        self
    }

    pub(crate) fn above_zero_seq<T, I>(mut self, values: I, field: &'static str) -> Self
    where
        T: Default + PartialOrd,
        I: IntoIterator<Item = T>,
    {
        for value in values {
            if value <= T::default() {
                self.push(Reason::NotGreaterThanZero, Some(field));
                break;
            }
        }
        self
    }

    pub(crate) fn min_max(mut self, range: NumberRange) -> Self {
        let ok = match range {
            NumberRange::Float { min, max } => {
                self = self.finite(min, "min").finite(max, "max");
                !min.is_finite() || !max.is_finite() || min <= max
            }
            NumberRange::Integer { min, max } => min <= max,
            NumberRange::Date { min, max } => min <= max,
            NumberRange::DateTime { min, max } => min <= max,
        };
        if !ok {
            self.push(Reason::MinMaxOutOfOrder(range), Some("range"));
        }
        self
    }

    pub(crate) fn unique<T: Eq + Hash + Debug + Copy>(
        mut self,
        values: impl IntoIterator<Item = T>,
        field: &'static str,
        is_error: bool,
    ) -> Self {
        let mut seen = HashMap::new();
        for value in values {
            let count = seen.entry(value).or_insert(0_usize);
            if *count == 1 {
                if is_error {
                    self.push(Reason::NotUnique(format!("{value:?}")), Some(field));
                } else {
                    self.push(Reason::SoftNotUnique(format!("{value:?}")), Some(field));
                }
            }
            *count += 1;
        }
        self
    }

    pub(crate) fn unit_vector(mut self, [x, y, z]: Vector3, field: &'static str) -> Self {
        const THRESHOLD: f64 = 1e-6;
        let mag2 = x * x + y * y + z * z;
        if (1.0 - mag2).abs() >= THRESHOLD {
            let len = (mag2.sqrt() * 1e7).floor() / 1e7;
            self.push(Reason::NotUnitVector([x, y, z], len), Some(field));
        }
        self
    }

    pub(crate) fn vectors_ortho2(mut self, u: Vector3, v: Vector3) -> Self {
        if !ortho(u, v) {
            self.push(Reason::NotOrthogonal(u, v), None);
        }
        self
    }

    pub(crate) fn vectors_ortho3(mut self, u: Vector3, v: Vector3, w: Vector3) -> Self {
        for (a, b) in [(u, v), (u, w), (v, w)] {
            if !ortho(u, v) {
                self.push(Reason::NotOrthogonal(a, b), None);
                break;
            }
        }
        self
    }

    pub(crate) fn array_size(mut self, size: u64, required: u64, field: &'static str) -> Self {
        if size != required {
            self.push(Reason::AttrLengthMismatch(size, required), Some(field));
        }
        self
    }

    pub(crate) fn array_size_opt(
        self,
        size_opt: Option<u64>,
        required: u64,
        field: &'static str,
    ) -> Self {
        if let Some(size) = size_opt {
            self.array_size(size, required, field)
        } else {
            self
        }
    }

    pub(crate) fn attrs_on_geometry(mut self, attrs: &Vec<Attribute>, geometry: &Geometry) -> Self {
        for attr in attrs {
            if matches!(attr.data, AttributeData::ProjectedTexture { .. })
                != (attr.location == Location::Projected)
            {
                self.push_full(
                    Reason::AttrLocationWrongForAttr(attr.location, attr.data.type_name()),
                    "Attribute",
                    Some("location"),
                    Some(&attr.name),
                );
            } else if let Some(geom_len) = geometry.location_len(attr.location) {
                if geom_len != attr.len() {
                    self.push_full(
                        Reason::AttrLengthMismatch(attr.len(), geom_len),
                        "Attribute",
                        None,
                        Some(&attr.name),
                    );
                }
            } else {
                self.push_full(
                    Reason::AttrLocationWrongForGeom(attr.location, geometry.type_name()),
                    "Attribute",
                    Some("location"),
                    Some(&attr.name),
                );
            }
        }
        self
    }

    pub(crate) fn attrs_on_attribute(mut self, attrs: &Vec<Attribute>, n_categories: u64) -> Self {
        for attr in attrs {
            if attr.location != Location::Categories {
                self.push_full(
                    Reason::AttrLocationWrongForGeom(attr.location, "AttributeData::Categories"),
                    "Attribute",
                    Some("location"),
                    Some(&attr.name),
                );
            } else if attr.len() != n_categories {
                self.push_full(
                    Reason::AttrLengthMismatch(attr.len(), n_categories),
                    "Attribute",
                    None,
                    Some(&attr.name),
                );
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, PointSet, array_type};

    use super::*;

    /// Test that if you have only warnings the result is `Ok`.
    #[test]
    fn problems_into_result() {
        let mut problems = Problems::default();
        problems.push(
            Reason::SoftNotUnique("x".to_owned()),
            "Test",
            Some("field"),
            None,
        );
        assert_eq!(problems.into_result().unwrap().len(), 1);
    }

    #[test]
    fn validator_basics() {
        let mut v = Validator::new().enter("Test");
        v.push(Reason::NotFinite, None);
        v.push(Reason::NotFinite, Some("field"));
        v = v.name("name");
        v.push(Reason::NotFinite, None);
        v.push(Reason::NotFinite, Some("field"));
        let errors: Vec<_> = v
            .finish()
            .into_iter()
            .map(|prob| prob.to_string())
            .collect();
        assert_eq!(
            errors,
            vec![
                "Error: 'Test' must be finite",
                "Error: 'Test::field' must be finite",
                "Error: 'Test' must be finite, inside 'name'",
                "Error: 'Test::field' must be finite, inside 'name'",
            ]
        )
    }

    #[test]
    fn validator_checks() {
        let attrs = vec![
            Attribute::new(
                "a",
                Location::Vertices,
                AttributeData::Number {
                    values: Array::new("1.parquet".to_owned(), 100).into(),
                    colormap: None,
                },
            ),
            Attribute::new(
                "b",
                Location::Primitives, // location error
                AttributeData::Number {
                    values: Array::new("2.parquet".to_owned(), 100).into(),
                    colormap: None,
                },
            ),
            Attribute::new(
                "c",
                Location::Vertices,
                AttributeData::Number {
                    values: Array::new("3.parquet".to_owned(), 101).into(), // length error
                    colormap: None,
                },
            ),
            Attribute::new(
                "c",
                Location::Vertices, // error
                AttributeData::ProjectedTexture {
                    orient: Default::default(),
                    width: 10.0,
                    height: 10.0,
                    image: Array::new("4.jpeg".to_owned(), 100),
                },
            ),
            Attribute::new(
                "d",
                Location::Projected,
                AttributeData::ProjectedTexture {
                    orient: Default::default(),
                    width: 10.0,
                    height: 10.0,
                    image: Array::new("6.png".to_owned(), 100), // missing file error
                },
            ),
        ];
        let results: Vec<_> = Validator::new()
            .with_filenames(["1.parquet"])
            .enter("Test")
            .finite(0.0, "zero")
            .finite(f64::INFINITY, "inf") // error
            .finite(f64::NAN, "nan") // error
            .finite_seq([0.0, f64::NEG_INFINITY, f64::NAN], "seq") // error
            .above_zero_seq([1.0, 2.0, 3.0], "normal")
            .above_zero_seq([1.0, 0.0, -1.0], "seq") // error
            .above_zero_seq([1.0, f64::NAN], "seq_nan")
            .min_max(NumberRange::Float {
                // error
                min: f64::NAN,
                max: 100.0,
            })
            .min_max(NumberRange::Float {
                min: 100.0,
                max: 100.0,
            })
            .min_max(NumberRange::Float {
                min: 101.5,
                max: 100.0,
            }) // error
            .unit_vector([1.0, 0.0, 0.0], "i")
            .unit_vector([0.5 * 2.0_f64.sqrt(), 0.5 * 2.0_f64.sqrt(), 0.0], "angled")
            .unit_vector([0.5, 0.0, 0.0], "short") // error
            .vectors_ortho2([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .vectors_ortho2([0.8, 0.0, 0.0], [0.0, 0.0, 1.0])
            .vectors_ortho2([0.0, 1.0, 0.0], [0.0, 0.0, -1.0])
            .vectors_ortho2([1.0, 0.0, 0.0], [0.8, 0.2, 0.0]) // error
            .vectors_ortho3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0])
            .vectors_ortho3([1.0, 0.001, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]) // error
            .attrs_on_geometry(
                &attrs,
                &PointSet::new(Array::new("5.parquet".to_owned(), 100)).into(),
            ) // 3 errors
            .array(
                &mut Array::<array_type::Text>::new("1.parquet".to_owned(), 10),
                array::Constraint::String,
                "fine",
            )
            .array(
                // error
                &mut Array::<array_type::Text>::new("2.parquet".to_owned(), 10),
                array::Constraint::String,
                "missing",
            )
            .subblock_mode_and_count(None, [16, 8, 15])
            .subblock_mode_and_count(Some(SubblockMode::Full), [16, 8, 5])
            .subblock_mode_and_count(Some(SubblockMode::Octree), [16, 8, 5]) // error
            .subblock_mode_and_count(Some(SubblockMode::Octree), [16, 8, 4])
            .unique([0; 0], "empty", true)
            .unique([1], "single", true)
            .unique([1, 2, 3, 4], "normal", false)
            .unique([1, 2, 3, 4, 2], "dupped", true) // warning
            .unique(["a", "b", "c", "d", "c", "a", "a"], "multiple", false) // 2 warnings
            .finish()
            .into_iter()
            .map(|p| p.to_string())
            .collect();
        let mut expected = vec![
            "Error: 'Test::inf' must be finite",
            "Error: 'Test::nan' must be finite",
            "Error: 'Test::seq' must be finite",
            "Error: 'Test::seq' must be greater than zero",
            "Error: 'Test::min' must be finite",
            "Error: 'Test::range' minimum is greater than maximum in [101.5, 100]",
            "Error: 'Test::short' must be a unit vector but [0.5, 0.0, 0.0] length is 0.5",
            "Error: 'Test' vectors are not orthogonal: [1.0, 0.0, 0.0] [0.8, 0.2, 0.0]",
            "Error: 'Test' vectors are not orthogonal: [1.0, 0.001, 0.0] [0.0, 1.0, 0.0]",
            "Error: 'Attribute::location' is Primitives which is not valid on PointSet geometry, inside 'b'",
            "Error: 'Attribute' length 101 does not match geometry (100), inside 'c'",
            "Error: 'Attribute::location' is Vertices which is not valid on ProjectedTexture attributes, inside 'c'",
            "Error: 'Test::missing' refers to non-existent archive member '2.parquet'",
            "Error: 'Test' sub-block counts [16, 8, 5] must be powers of two for octree mode",
            "Error: 'Test::dupped' must be unique but 2 is repeated",
            "Warning: 'Test::multiple' contains duplicate of \"c\"",
            "Warning: 'Test::multiple' contains duplicate of \"a\"",
        ];
        let mut unexpected = Vec::new();
        for s in results {
            if let Some(index) = expected.iter().position(|e| *e == &s) {
                expected.remove(index);
            } else {
                unexpected.push(s.to_owned());
            }
        }
        if !unexpected.is_empty() || !expected.is_empty() {
            panic!("unexpected problems: {unexpected:#?}\nexpected but not found: {expected:#?}");
        }
    }
}
