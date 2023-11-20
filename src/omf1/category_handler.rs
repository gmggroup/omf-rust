use core::panic;
use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::{error::Error, file::Writer};

use super::{
    array::{load_scalars, ScalarValues},
    model::{LegendArrayModel, LegendArrays},
    objects::{ColorArray, DateTimeArray, Key, ScalarArray, StringArray},
    reader::Omf1Reader,
};

struct Legend {
    name: String,
    description: String,
    data: Data,
}

enum Data {
    Color(Vec<[u8; 3]>),
    DateTime(Vec<Option<DateTime<Utc>>>),
    String(Vec<String>),
    Float(Vec<f64>),
    Int(Vec<i64>),
}

impl Data {
    fn len(&self) -> usize {
        match self {
            Data::Color(x) => x.len(),
            Data::DateTime(x) => x.len(),
            Data::String(x) => x.len(),
            Data::Float(x) => x.len(),
            Data::Int(x) => x.len(),
        }
    }

    fn names_score(&self) -> Option<(usize, usize)> {
        if let Self::String(strings) = self {
            let unique_count = strings
                .iter()
                .filter(|s| !s.is_empty())
                .collect::<HashSet<_>>()
                .len();
            let total_len = strings.iter().map(|s| s.len()).sum();
            Some((unique_count, total_len))
        } else {
            None
        }
    }

    fn gradient_score(&self) -> Option<usize> {
        if let Self::Color(colors) = self {
            Some(colors.iter().collect::<HashSet<_>>().len())
        } else {
            None
        }
    }
}

impl Legend {
    fn write(self, w: &mut Writer, len: usize) -> Result<crate::Attribute, Error> {
        let data = match self.data {
            Data::Color(colors) => crate::AttributeData::Color {
                values: w.array_colors(iter_to_len(
                    len,
                    colors.into_iter().map(|[r, g, b]| Some([r, g, b, u8::MAX])),
                    None,
                ))?,
            },
            Data::DateTime(date_times) => crate::AttributeData::Number {
                values: w.array_numbers(iter_to_len(len, date_times.into_iter(), None))?,
                colormap: None,
            },
            Data::String(strings) => crate::AttributeData::Text {
                values: w.array_text(iter_to_len(
                    len,
                    strings
                        .into_iter()
                        .map(|s| if s.is_empty() { None } else { Some(s) }),
                    None,
                ))?,
            },
            Data::Float(numbers) => crate::AttributeData::Number {
                values: w.array_numbers(iter_to_len(
                    len,
                    numbers
                        .into_iter()
                        .map(|x| if x.is_nan() { None } else { Some(x) }),
                    None,
                ))?,
                colormap: None,
            },
            Data::Int(numbers) => crate::AttributeData::Number {
                values: w.array_numbers(iter_to_len(len, numbers.into_iter().map(Some), None))?,
                colormap: None,
            },
        };
        Ok(crate::Attribute {
            name: self.name,
            description: self.description,
            units: Default::default(),
            metadata: Default::default(),
            location: crate::Location::Categories,
            data,
        })
    }
}

pub struct CategoryHandler {
    max_index: u32,
    len: usize,
    legends: Vec<Legend>,
    names: Vec<String>,
    gradient: Option<Vec<[u8; 3]>>,
}

impl CategoryHandler {
    pub fn new(max_index: u32) -> Self {
        Self {
            max_index,
            len: 0,
            legends: Vec::new(),
            names: Vec::new(),
            gradient: None,
        }
    }

    pub fn add(
        &mut self,
        r: &Omf1Reader,
        name: &str,
        description: &str,
        key: &Key<LegendArrays>,
    ) -> Result<(), Error> {
        let data = match r.model(key)? {
            LegendArrayModel::ColorArray(ColorArray { array, .. }) => Data::Color(array.clone()),
            LegendArrayModel::DateTimeArray(DateTimeArray { array, .. }) => {
                Data::DateTime(array.clone())
            }
            LegendArrayModel::StringArray(StringArray { array, .. }) => Data::String(array.clone()),
            LegendArrayModel::ScalarArray(a @ ScalarArray { .. }) => match load_scalars(r, a)? {
                ScalarValues::Float(x) => Data::Float(x),
                ScalarValues::Int(x) => Data::Int(x),
            },
        };
        self.legends.push(Legend {
            name: name.to_owned(),
            description: description.to_owned(),
            data,
        });
        Ok(())
    }

    fn process(&mut self) {
        self.len = self
            .legends
            .iter()
            .map(|l| l.data.len())
            .max()
            .unwrap_or(0)
            .max(self.max_index as usize);
        if let Some((_, index)) = self
            .legends
            .iter()
            .enumerate()
            .filter_map(|(i, l)| l.data.names_score().map(|score| (score, i)))
            .max()
        {
            let Legend {
                data: Data::String(names),
                ..
            } = self.legends.remove(index)
            else {
                panic!("expected string legend");
            };
            self.names = names;
        } else {
            self.names = (0..self.len).map(|i| format!("Category{i}")).collect();
        }
        if let Some((_, index)) = self
            .legends
            .iter()
            .enumerate()
            .filter_map(|(i, l)| l.data.gradient_score().map(|score| (score, i)))
            .max()
        {
            let Legend {
                data: Data::Color(colors),
                ..
            } = self.legends.remove(index)
            else {
                panic!("expected color legend");
            };
            self.gradient = Some(colors);
        }
    }

    pub fn write(
        mut self,
        w: &mut Writer,
        values: crate::Array<crate::array_type::Index>,
    ) -> Result<crate::AttributeData, Error> {
        self.process();
        Ok(crate::AttributeData::Category {
            values,
            names: w.array_names(iter_to_len(self.len, self.names.into_iter(), String::new()))?,
            gradient: self
                .gradient
                .map(|g| {
                    w.array_gradient(iter_to_len(
                        self.len,
                        g.into_iter().map(|[r, g, b]| [r, g, b, u8::MAX]),
                        [128, 128, 128, 255],
                    ))
                })
                .transpose()?,
            attributes: self
                .legends
                .into_iter()
                .map(|l| l.write(w, self.len))
                .collect::<Result<_, _>>()?,
        })
    }
}

pub fn iter_to_len<T: Clone>(
    len: usize,
    input: impl Iterator<Item = T>,
    pad: T,
) -> impl Iterator<Item = T> {
    let padding = Some(pad).into_iter().cycle();
    input.chain(padding).take(len)
}
