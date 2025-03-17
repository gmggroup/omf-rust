use parquet::{
    basic::{Compression, Encoding, GzipLevel},
    errors::ParquetError,
    file::{
        properties::{
            DEFAULT_STATISTICS_ENABLED, EnabledStatistics, WriterProperties, WriterVersion,
        },
        writer::SerializedFileWriter,
    },
    schema::types::Type,
};

use crate::error::Error;

use super::{array_type::PqArrayType, source::*};

#[derive(Debug, Clone)]
pub struct PqWriteOptions {
    pub row_group_size: usize,
    pub compression_level: u32,
    pub statistics: bool,
}

impl Default for PqWriteOptions {
    fn default() -> Self {
        Self {
            // Small row groups are massively slower.
            row_group_size: 1024 * 1024,
            // Default gzip compression level.
            compression_level: 6,
            // Statistics just waste time because we don't read them.
            statistics: false,
        }
    }
}

impl PqWriteOptions {
    fn properties(&self) -> WriterProperties {
        WriterProperties::builder()
            .set_writer_version(WriterVersion::PARQUET_2_0)
            .set_compression(Compression::GZIP(
                GzipLevel::try_new(self.compression_level.clamp(0, 10))
                    .expect("valid compression level"),
            ))
            .set_statistics_enabled(if self.statistics {
                DEFAULT_STATISTICS_ENABLED
            } else {
                EnabledStatistics::None
            })
            .set_max_row_group_size(self.row_group_size)
            // We don't need any fancy encodings, and not using them is faster.
            .set_dictionary_enabled(false)
            .set_encoding(Encoding::PLAIN)
            .build()
    }
}

#[derive(Default)]
pub struct PqArrayWriter<'a> {
    options: PqWriteOptions,
    sources: Vec<Box<dyn Source + 'a>>,
    total_written: u64,
}

impl<'a> PqArrayWriter<'a> {
    pub fn new(options: PqWriteOptions) -> Self {
        Self {
            options,
            sources: Vec::new(),
            total_written: 0,
        }
    }

    fn push_source(&mut self, source: impl Source + 'a) {
        self.sources.push(Box::new(source))
    }

    pub fn add<P: PqArrayType, I: IntoIterator<Item = P> + 'a>(
        &mut self,
        name: &str,
        data: I,
    ) -> Result<(), Error> {
        self.push_source(RowSource::new(
            &[name],
            data.into_iter().map(|item| (item,)),
            self.options.row_group_size,
        )?);
        Ok(())
    }

    pub fn add_nullable<P: PqArrayType, I: IntoIterator<Item = Option<P>> + 'a>(
        &mut self,
        name: &str,
        data: I,
    ) -> Result<(), Error> {
        self.push_source(NullableRowSource::new_single(
            name,
            data.into_iter()
                .map(|opt_item| opt_item.map(|item| (item,))),
            self.options.row_group_size,
        )?);
        Ok(())
    }

    pub fn add_multiple<R, I>(&mut self, names: &[&str], data: I) -> Result<(), Error>
    where
        R: PqArrayRow,
        I: IntoIterator<Item = R> + 'a,
    {
        self.push_source(RowSource::new(
            names,
            data.into_iter(),
            self.options.row_group_size,
        )?);
        Ok(())
    }

    pub fn add_nullable_group<R: PqArrayRow, I: IntoIterator<Item = Option<R>> + 'a>(
        &mut self,
        group_name: &str,
        field_names: &[&str],
        data: I,
    ) -> Result<(), Error> {
        self.push_source(NullableRowSource::new(
            group_name,
            field_names,
            data.into_iter(),
            self.options.row_group_size,
        )?);
        Ok(())
    }

    fn schema(&self) -> Result<Type, Error> {
        let fields = self
            .sources
            .iter()
            .flat_map(|s| s.types())
            .map(Into::into)
            .collect();
        Type::group_type_builder("schema")
            .with_fields(fields)
            .build()
            .map_err(Into::into)
    }

    fn check_counts(&mut self, counts: &[u64]) -> Result<bool, Error> {
        let (min, max) = counts
            .iter()
            .fold((u64::MAX, 0), |(a, b), &c| (a.min(c), b.max(c)));
        self.total_written += min;
        if min != max {
            return Err(ParquetError::General(format!(
                "uneven iterator lengths after {} items",
                self.total_written
            ))
            .into());
        }
        Ok(min == 0)
    }

    pub fn write(mut self, w: impl std::io::Write + Send) -> Result<u64, Error> {
        // Create Parquet writer.
        let mut writer =
            SerializedFileWriter::new(w, self.schema()?.into(), self.options.properties().into())?;
        // Write data.
        let mut counts = vec![0_u64; self.sources.len()];
        loop {
            // Buffer data, collecting counts.
            for (source, n) in self.sources.iter_mut().zip(&mut counts) {
                *n = source.buffer(self.options.row_group_size) as u64;
            }
            // Check that sources buffered the same amount.
            if self.check_counts(&counts)? {
                break;
            }
            // Write that data.
            let mut row_group = writer.next_row_group()?;
            for source in &mut self.sources {
                source.write(&mut row_group)?;
            }
            row_group.close()?;
        }
        writer.close()?;
        Ok(self.total_written)
    }
}
