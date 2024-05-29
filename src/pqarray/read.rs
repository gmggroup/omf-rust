use std::{fmt::Debug, marker::PhantomData, ops::Range, sync::Arc};

use parquet::{
    basic::{LogicalType, Repetition},
    column::reader::{ColumnReader, ColumnReaderImpl},
    data_type::DataType,
    errors::ParquetError,
    file::{
        metadata::ParquetMetaData,
        reader::{ChunkReader, FileReader},
        serialized_reader::SerializedFileReader,
    },
    schema::types::{ColumnPath, Type},
};

use crate::error::Error;

use super::{array_type::PqArrayType, PqArrayMatcher};

const CHUNK_SIZE: usize = 8192;

#[derive(Debug, Clone)]
struct Info {
    column_index: usize,
    size_hint: (usize, Option<usize>),
    logical_type: Option<LogicalType>,
}

fn invalid(message: impl Into<String>) -> Error {
    ParquetError::General(message.into()).into()
}

/// Checks the type of a column so it at least shouldn't panic.
/// For better errors use [`PqArrayMatcher`](super::PqArrayMatcher) before reading.
fn check<P: PqArrayType>(
    metadata: &ParquetMetaData,
    path: ColumnPath,
    nullable: bool,
) -> Result<Info, Error> {
    let schema = metadata.file_metadata().schema_descr();
    // Find column.
    let column_index = schema
        .columns()
        .iter()
        .enumerate()
        .find_map(|(i, c)| if c.path() == &path { Some(i) } else { None })
        .ok_or_else(|| invalid(format!("Parquet column not found: {}", path)))?;
    // Check primitive and logical types.
    let ty = schema.column(column_index).self_type_ptr();
    if !ty.is_primitive()
        || ty.get_physical_type() != P::physical_type()
        || !P::check_logical_type(ty.get_basic_info().logical_type())
    {
        return Err(invalid("column type mismatch"));
    }
    // Check repetition.
    match (ty.get_basic_info().repetition(), nullable) {
        (Repetition::REQUIRED, true | false) => (),
        (Repetition::OPTIONAL, true) => (),
        _ => return Err(invalid("column repetition mismatch")),
    }
    Ok(Info {
        column_index,
        logical_type: ty.get_basic_info().logical_type(),
        size_hint: if let Ok(n) = metadata.file_metadata().num_rows().try_into() {
            (n, Some(n))
        } else {
            (usize::MAX, None)
        },
    })
}

#[derive(Debug, Default)]
struct Counter(usize);

impl Counter {
    fn new() -> Self {
        Self(0)
    }

    fn incr(&mut self) -> usize {
        let out = self.0;
        self.0 += 1;
        out
    }

    fn reset(&mut self) {
        self.0 = 0;
    }

    fn reached(&self, limit: usize) -> bool {
        self.0 >= limit
    }
}

/// Reads a block of data from a column reader.
fn read_column_chunk<D: DataType>(
    column: &mut ColumnReaderImpl<D>,
    values: &mut Vec<D::T>,
    mut def_levels: Option<&mut Vec<i16>>,
) -> Result<(usize, usize), Error> {
    values.clear();
    let mut max_records = values.capacity();
    if let Some(d) = &mut def_levels {
        d.clear();
        max_records = max_records.max(d.capacity());
    }
    let (n_val, n_def, _n_rep) = column.read_records(max_records, def_levels, None, values)?;
    Ok((n_val, n_def))
}

/// Iterates over all the values in one column within one row group.
///
/// Two implementations support required and nullable values.
pub trait GroupValues<P: PqArrayType> {
    type Item;
    const NULLABLE: bool;

    fn new(logical_type: Option<LogicalType>) -> Self;
    fn set_column_reader(&mut self, column: Result<ColumnReaderImpl<P::DataType>, Error>);
    fn next(&mut self) -> Option<Result<Self::Item, Error>>;
}

pub struct RequiredGroupValues<P: PqArrayType> {
    column: Result<ColumnReaderImpl<P::DataType>, Option<Error>>,
    len: usize,
    index: Counter,
    values: Vec<<P::DataType as DataType>::T>,
    logical_type: Option<LogicalType>,
}

impl<P: PqArrayType> GroupValues<P> for RequiredGroupValues<P> {
    type Item = P;
    const NULLABLE: bool = false;

    fn new(logical_type: Option<LogicalType>) -> Self {
        Self {
            column: Err(None),
            len: 0,
            index: Counter::new(),
            values: vec![Default::default(); CHUNK_SIZE],
            logical_type,
        }
    }

    fn set_column_reader(&mut self, column: Result<ColumnReaderImpl<P::DataType>, Error>) {
        self.column = column.map_err(Some);
        self.len = 0;
        self.index.reset();
    }

    fn next(&mut self) -> Option<Result<P, Error>> {
        let column = match &mut self.column {
            Ok(x) => x,
            Err(None) => return None,
            Err(e @ Some(_)) => return Some(Err(e.take().unwrap())),
        };
        if self.index.reached(self.len) {
            match read_column_chunk(column, &mut self.values, None) {
                Ok((n_read, _)) => {
                    self.len = n_read;
                    self.index.reset();
                    if n_read == 0 {
                        return None;
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
        let value = self.values[self.index.incr()].clone();
        Some(Ok(P::from_parquet(value, &self.logical_type)))
    }
}

/// Iterates over all the values in one column within one row group.
pub struct NullableGroupValues<P: PqArrayType> {
    column: Result<ColumnReaderImpl<P::DataType>, Option<Error>>,
    len: usize,
    index: Counter,
    value_index: Counter,
    values: Vec<<P::DataType as DataType>::T>,
    def_levels: Vec<i16>,
    logical_type: Option<LogicalType>,
}

impl<P: PqArrayType> GroupValues<P> for NullableGroupValues<P> {
    type Item = Option<P>;
    const NULLABLE: bool = true;

    fn new(logical_type: Option<LogicalType>) -> Self {
        Self {
            column: Err(None),
            len: 0,
            index: Counter::new(),
            value_index: Counter::new(),
            values: Vec::with_capacity(CHUNK_SIZE),
            def_levels: Vec::with_capacity(CHUNK_SIZE),
            logical_type,
        }
    }

    fn set_column_reader(&mut self, column: Result<ColumnReaderImpl<P::DataType>, Error>) {
        self.column = column.map_err(Some);
        self.len = 0;
        self.index.reset();
        self.value_index.reset();
    }

    fn next(&mut self) -> Option<Result<Option<P>, Error>> {
        let column = match &mut self.column {
            Ok(x) => x,
            Err(None) => return None,
            Err(e @ Some(_)) => return Some(Err(e.take().unwrap())),
        };
        if self.index.reached(self.len) {
            match read_column_chunk(column, &mut self.values, Some(&mut self.def_levels)) {
                Ok((n_read, _)) => {
                    self.len = n_read;
                    self.index.reset();
                    self.value_index.reset();
                    if n_read == 0 {
                        return None;
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
        let value = if self.def_levels[self.index.incr()] == 0 {
            None
        } else {
            Some(P::from_parquet(
                self.values[self.value_index.incr()].clone(),
                &self.logical_type,
            ))
        };
        Some(Ok(value))
    }
}

/// Iterates over row groups in a file, generating column readers.
struct Groups<R: ChunkReader + 'static> {
    file_reader: Arc<SerializedFileReader<R>>,
    column_index: usize,
    indices: Range<usize>,
}

impl<R: ChunkReader + 'static> Iterator for Groups<R> {
    type Item = Result<ColumnReader, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.next()?;
        Some(
            self.file_reader
                .get_row_group(index)
                .and_then(|rg| rg.get_column_reader(self.column_index))
                .map_err(Into::into),
        )
    }
}

/// Iterates over a Parquet file object, generating all the values in a single column.
pub struct ColumnIter<P: PqArrayType, R: ChunkReader + 'static, G: GroupValues<P>> {
    info: Info,
    first: bool,
    groups: Groups<R>,
    group_values: G,
    _phantom: PhantomData<P>,
}

impl<P: PqArrayType, R: ChunkReader + 'static, G: GroupValues<P>> ColumnIter<P, R, G> {
    fn new(file_reader: Arc<SerializedFileReader<R>>, path: ColumnPath) -> Result<Self, Error> {
        let info = check::<P>(file_reader.metadata(), path, G::NULLABLE)?;
        Ok(Self {
            first: true,
            groups: Groups {
                indices: 0..file_reader.num_row_groups(),
                file_reader,
                column_index: info.column_index,
            },
            group_values: G::new(info.logical_type.clone()),
            info,
            _phantom: PhantomData,
        })
    }

    fn advance_group(&mut self) -> Option<()> {
        self.group_values.set_column_reader(
            self.groups
                .next()?
                .map(|c| P::DataType::get_column_reader(c).expect("matching column reader type")),
        );
        Some(())
    }
}

impl<P: PqArrayType, R: ChunkReader + 'static, G: GroupValues<P>> Iterator for ColumnIter<P, R, G> {
    type Item = Result<G::Item, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            self.advance_group()?;
        }
        loop {
            if let Some(value) = self.group_values.next() {
                return Some(value);
            }
            self.advance_group()?;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.info.size_hint
    }
}

impl<P: PqArrayType, R: ChunkReader + 'static, G: GroupValues<P>> Debug for ColumnIter<P, R, G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnIter")
            .field("info", &self.info)
            .field("first", &self.first)
            .finish()
    }
}

/// Combines multiple column iterators into one.
#[derive(Debug)]
pub struct ColumnZip<T, I, const N: usize> {
    iters: [I; N],
    _phantom: PhantomData<T>,
}

impl<T, I, const N: usize> Iterator for ColumnZip<T, I, N>
where
    T: Default,
    I: Iterator<Item = Result<T, Error>>,
{
    type Item = Result<[T; N], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let items: [_; N] = std::array::from_fn(|i| self.iters[i].next());
        let mut output = std::array::from_fn(|_| T::default());
        for (item, out) in items.into_iter().zip(&mut output) {
            match item {
                None => return None,
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(value)) => *out = value,
            }
        }
        Some(Ok(output))
    }
}

/// Combines multiple nullable column iterators into one.
#[derive(Debug)]
pub struct NullableColumnZip<T, I, const N: usize> {
    iters: [I; N],
    _phantom: PhantomData<T>,
}

impl<T, I, const N: usize> Iterator for NullableColumnZip<T, I, N>
where
    T: Default,
    I: Iterator<Item = Result<Option<T>, Error>>,
{
    type Item = Result<Option<[T; N]>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let items: [_; N] = std::array::from_fn(|i| self.iters[i].next());
        let mut output = std::array::from_fn(|_| T::default());
        for (item, out) in items.into_iter().zip(&mut output) {
            match item {
                None => return None,
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(None)) => return Some(Ok(None)),
                Some(Ok(Some(value))) => *out = value,
            }
        }
        Some(Ok(Some(output)))
    }
}

/// Provides an interface for reading OMF arrays from Parquet files.
pub struct PqArrayReader<R: ChunkReader + 'static> {
    file_reader: Arc<SerializedFileReader<R>>,
}

impl<R: ChunkReader + 'static> PqArrayReader<R> {
    pub fn new(read: R) -> Result<Self, Error> {
        Ok(Self {
            file_reader: SerializedFileReader::new(read)?.into(),
        })
    }

    pub fn len(&self) -> u64 {
        self.file_reader
            .metadata()
            .file_metadata()
            .num_rows()
            .try_into()
            .unwrap_or_default()
    }

    pub fn matches<T: Copy>(&self, matcher: &PqArrayMatcher<T>) -> Result<T, Error> {
        matcher.check(self.schema())
    }

    pub fn schema(&self) -> Arc<Type> {
        self.file_reader
            .metadata()
            .file_metadata()
            .schema_descr()
            .root_schema_ptr()
    }

    pub fn iter_column<P: PqArrayType>(&self, name: &str) -> Result<SimpleIter<P, R>, Error> {
        ColumnIter::new(
            self.file_reader.clone(),
            ColumnPath::new(vec![name.to_owned()]),
        )
    }

    pub fn iter_nullable_column<P: PqArrayType>(
        &self,
        name: &str,
    ) -> Result<NullableIter<P, R>, Error> {
        ColumnIter::new(
            self.file_reader.clone(),
            ColumnPath::new(vec![name.to_owned()]),
        )
    }

    pub fn iter_multi_column<P: PqArrayType, const N: usize>(
        &self,
        names: [&str; N],
    ) -> Result<MultiIter<P, R, N>, Error> {
        let iters: [_; N] = names
            .iter()
            .map(|n| self.iter_column::<P>(n))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .expect("correct length");
        Ok(ColumnZip {
            iters,
            _phantom: PhantomData,
        })
    }

    pub fn iter_nullable_group_column<P: PqArrayType, const N: usize>(
        &self,
        group_name: &str,
        field_names: [&str; N],
    ) -> Result<NullableGroupIter<P, R, N>, Error> {
        let iters: [_; N] = field_names
            .into_iter()
            .map(|name| {
                ColumnIter::new(
                    self.file_reader.clone(),
                    ColumnPath::new(vec![group_name.to_owned(), name.to_owned()]),
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .expect("correct length");
        Ok(NullableColumnZip {
            iters,
            _phantom: PhantomData,
        })
    }
}

pub type SimpleIter<P, R> = ColumnIter<P, R, RequiredGroupValues<P>>;
pub type NullableIter<P, R> = ColumnIter<P, R, NullableGroupValues<P>>;
pub type MultiIter<P, R, const N: usize> =
    ColumnZip<P, ColumnIter<P, R, RequiredGroupValues<P>>, N>;
pub type NullableGroupIter<P, R, const N: usize> =
    NullableColumnZip<P, ColumnIter<P, R, NullableGroupValues<P>>, N>;
