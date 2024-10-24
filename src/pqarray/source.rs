use std::{collections::TryReserveError, io::Write};

use parquet::{
    basic::Repetition,
    data_type::DataType,
    errors::ParquetError,
    file::writer::{SerializedColumnWriter, SerializedRowGroupWriter},
    schema::types::Type,
};

use super::array_type::PqArrayType;

pub trait RowGrouper {
    fn next_column(&mut self) -> Result<Option<SerializedColumnWriter<'_>>, ParquetError>;
}

impl<W: Write + Send> RowGrouper for SerializedRowGroupWriter<'_, W> {
    fn next_column(&mut self) -> Result<Option<SerializedColumnWriter<'_>>, ParquetError> {
        SerializedRowGroupWriter::next_column(self)
    }
}

pub trait Source {
    /// Parquet schema types that this source writes.
    fn types(&self) -> Vec<Type>;
    /// Read up to `size` items into internal buffers, returning the number of items read.
    fn buffer(&mut self, size: usize) -> usize;
    /// Write data from the last `buffer()` call.
    fn write(&mut self, row_group: &mut dyn RowGrouper) -> Result<(), ParquetError>;
}

fn single_type<P: PqArrayType>(name: &str, nullable: bool) -> Type {
    Type::primitive_type_builder(name, P::physical_type())
        .with_repetition(if nullable {
            Repetition::OPTIONAL
        } else {
            Repetition::REQUIRED
        })
        .with_logical_type(P::logical_type())
        .build()
        .expect("valid type")
}

fn row_group_vec<T>(n: usize) -> Result<Vec<T>, TryReserveError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)?;
    Ok(v)
}

fn row_group_vec_init<T: Copy>(value: T, n: usize) -> Result<Vec<T>, TryReserveError> {
    let mut v = row_group_vec(n)?;
    v.resize(n, value);
    Ok(v)
}

pub trait PqArrayRow: 'static {
    type Buffer: Sized;
    const WIDTH: usize;

    fn types(names: &[&str]) -> Vec<Type>;
    fn make_buffer(row_group_size: usize) -> Result<Self::Buffer, TryReserveError>;
    fn clear_buffer(buffer: &mut Self::Buffer);
    fn add_to_buffer(self, buffer: &mut Self::Buffer);
    fn write_buffer(
        buffer: &Self::Buffer,
        row_group: &mut dyn RowGrouper,
        def_levels: &[i16],
    ) -> Result<(), ParquetError>;
}

pub struct RowSource<R: PqArrayRow, I: Iterator<Item = R>> {
    row_types: Vec<Type>,
    iter: I,
    buffer: R::Buffer,
    def_levels: Vec<i16>,
    count: usize,
}

impl<R: PqArrayRow, I: Iterator<Item = R>> RowSource<R, I> {
    pub fn new(names: &[&str], iter: I, row_group_size: usize) -> Result<Self, TryReserveError> {
        Ok(Self {
            row_types: R::types(names),
            iter,
            buffer: R::make_buffer(row_group_size)?,
            def_levels: row_group_vec_init(1, row_group_size)?,
            count: 0,
        })
    }
}

impl<R: PqArrayRow, I: Iterator<Item = R>> Source for RowSource<R, I> {
    fn types(&self) -> Vec<Type> {
        self.row_types.clone()
    }

    fn buffer(&mut self, size: usize) -> usize {
        R::clear_buffer(&mut self.buffer);
        self.count = 0;
        for row in self.iter.by_ref().take(size) {
            row.add_to_buffer(&mut self.buffer);
            self.count += 1;
        }
        self.count
    }

    fn write(&mut self, row_group: &mut dyn RowGrouper) -> Result<(), ParquetError> {
        R::write_buffer(&self.buffer, row_group, &self.def_levels[..self.count])
    }
}

pub struct NullableRowSource<R: PqArrayRow, I: Iterator<Item = Option<R>>> {
    ty: Type,
    iter: I,
    buffer: R::Buffer,
    def_levels: Vec<i16>,
}

impl<R: PqArrayRow, I: Iterator<Item = Option<R>>> NullableRowSource<R, I> {
    pub fn new(
        group_name: &str,
        names: &[&str],
        iter: I,
        row_group_size: usize,
    ) -> Result<Self, TryReserveError> {
        Ok(Self {
            ty: Type::group_type_builder(group_name)
                .with_repetition(Repetition::OPTIONAL)
                .with_fields(R::types(names).into_iter().map(Into::into).collect())
                .build()
                .expect("valid type"),
            iter,
            buffer: R::make_buffer(row_group_size)?,
            def_levels: row_group_vec_init(1, row_group_size)?,
        })
    }

    pub fn new_single(name: &str, iter: I, row_group_size: usize) -> Result<Self, TryReserveError> {
        assert_eq!(R::WIDTH, 1);
        let ty = R::types(&["tmp"]).into_iter().next().unwrap();
        Ok(Self {
            ty: Type::primitive_type_builder(name, ty.get_physical_type())
                .with_repetition(Repetition::OPTIONAL)
                .with_logical_type(ty.get_basic_info().logical_type())
                .build()
                .expect("valid type"),
            iter,
            buffer: R::make_buffer(row_group_size)?,
            def_levels: row_group_vec_init(1, row_group_size)?,
        })
    }
}

impl<R: PqArrayRow, I: Iterator<Item = Option<R>>> Source for NullableRowSource<R, I> {
    fn types(&self) -> Vec<Type> {
        vec![self.ty.clone()]
    }

    fn buffer(&mut self, size: usize) -> usize {
        self.def_levels.clear();
        R::clear_buffer(&mut self.buffer);
        for opt_row in self.iter.by_ref().take(size) {
            if let Some(row) = opt_row {
                row.add_to_buffer(&mut self.buffer);
                self.def_levels.push(1);
            } else {
                self.def_levels.push(0);
            }
        }
        self.def_levels.len()
    }

    fn write(&mut self, row_group: &mut dyn RowGrouper) -> Result<(), ParquetError> {
        R::write_buffer(
            &self.buffer,
            row_group,
            &self.def_levels[..self.def_levels.len()],
        )
    }
}

impl<P: PqArrayType, const N: usize> PqArrayRow for [P; N] {
    type Buffer = [Vec<<P::DataType as DataType>::T>; N];
    const WIDTH: usize = N;

    fn types(names: &[&str]) -> Vec<Type> {
        assert_eq!(names.len(), N);
        names.iter().map(|n| single_type::<P>(n, false)).collect()
    }

    fn make_buffer(row_group_size: usize) -> Result<Self::Buffer, TryReserveError> {
        let mut buffer = std::array::from_fn(|_| Vec::new());
        for b in &mut buffer {
            *b = row_group_vec(row_group_size)?;
        }
        Ok(buffer)
    }

    fn clear_buffer(buffer: &mut Self::Buffer) {
        for b in buffer {
            b.clear();
        }
    }

    fn add_to_buffer(self, buffer: &mut Self::Buffer) {
        for (b, a) in buffer.iter_mut().zip(self.into_iter()) {
            b.push(a.to_parquet());
        }
    }

    fn write_buffer(
        buffer: &Self::Buffer,
        row_group: &mut dyn RowGrouper,
        def_levels: &[i16],
    ) -> Result<(), ParquetError> {
        for b in buffer {
            let mut column = row_group.next_column()?.expect("columns to match schema");
            column
                .typed::<P::DataType>()
                .write_batch(b, Some(def_levels), None)?;
            column.close()?;
        }
        Ok(())
    }
}

macro_rules! row {
    ($width:literal, { $($i:tt $P:ident),* }) => {
        impl<$( $P: PqArrayType ),*> PqArrayRow for ($( $P, )*) {
            type Buffer = (
                $( Vec<<$P::DataType as DataType>::T>, )*
            );
            const WIDTH: usize = $width;

            fn types(names: &[&str]) -> Vec<Type> {
                assert_eq!(names.len(), $width);
                let mut names_iter = names.into_iter();
                vec![$(
                    single_type::<$P>(names_iter.next().unwrap(), false),
                )*]
            }

            fn make_buffer(row_group_size: usize) -> Result<Self::Buffer, TryReserveError> {
                Ok(($(
                    row_group_vec::<<$P::DataType as DataType>::T>(row_group_size)?,
                )*))
            }

            fn clear_buffer(buffer: &mut Self::Buffer) {
                $( buffer.$i.clear(); )*
            }

            fn add_to_buffer(self, buffer: &mut Self::Buffer) {
                $( buffer.$i.push(self.$i.to_parquet()); )*
            }

            fn write_buffer(
                buffer: &Self::Buffer,
                row_group: &mut dyn RowGrouper,
                def_levels: &[i16],
            ) -> Result<(), ParquetError> {
                $(
                    let mut column = row_group.next_column()?.expect("columns to match schema");
                    column
                        .typed::<$P::DataType>()
                        .write_batch(&buffer.$i, Some(def_levels), None)?;
                    column.close()?;
                )*
                Ok(())
            }
        }
    };
}

row!(1, { 0 P0 });
row!(2, { 0 P0, 1 P1 });
row!(3, { 0 P0, 1 P1, 2 P2 });
row!(4, { 0 P0, 1 P1, 2 P2, 3 P3 });
row!(5, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4 });
row!(6, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4, 5 P5 });
row!(7, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4, 5 P5, 6 P6 });
row!(8, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4, 5 P5, 6 P6, 7 P7 });
row!(9, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4, 5 P5, 6 P6, 7 P7, 8 P8 });
row!(10, { 0 P0, 1 P1, 2 P2, 3 P3, 4 P4, 5 P5, 6 P6, 7 P7, 8 P8, 9 P9 });
