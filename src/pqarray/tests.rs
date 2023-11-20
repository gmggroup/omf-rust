use std::str::FromStr;

use bytes::Bytes;
use chrono::prelude::*;

use super::{schema::schema, *};

fn write_read(writer: PqArrayWriter) -> PqArrayReader<Bytes> {
    let mut buffer = Vec::new();
    writer.write(&mut buffer).unwrap();
    PqArrayReader::new(Bytes::from(buffer)).unwrap()
}

fn read_column<P: PqArrayType>(reader: &PqArrayReader<Bytes>, name: &str) -> Vec<P> {
    reader
        .iter_column::<P>(name)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn read_nullable_column<P: PqArrayType>(
    reader: &PqArrayReader<Bytes>,
    name: &str,
) -> Vec<Option<P>> {
    reader
        .iter_nullable_column::<P>(name)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn read_multi_column<P: PqArrayType, const N: usize>(
    reader: &PqArrayReader<Bytes>,
    names: [&str; N],
) -> Vec<[P; N]> {
    reader
        .iter_multi_column::<P, N>(names)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn read_nullable_group_column<P: PqArrayType, const N: usize>(
    reader: &PqArrayReader<Bytes>,
    group_name: &str,
    field_names: [&str; N],
) -> Vec<Option<[P; N]>> {
    reader
        .iter_nullable_group_column::<P, N>(group_name, field_names)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn parquet_array_bool() {
    let values = [true, true, false, true];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("filter", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required boolean filter;
        }
    );
    assert_eq!(read_column::<bool>(&reader, "filter"), values);
}

#[test]
fn parquet_array_f64() {
    let values = [0.0_f64, 1.0, 2.0, 3.0, 4.0];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("value", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required double value;
        }
    );
    assert_eq!(read_column::<f64>(&reader, "value"), values);
}

#[test]
fn parquet_array_u32() {
    let values = [0_u32, 1, 2, 3, 4_294_967_295];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("value", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int32 value(integer(32,false));
        }
    );
    assert_eq!(read_column::<u32>(&reader, "value"), values);
}

#[test]
fn parquet_array_u8() {
    let values = [0_u8, 1, 2, 3, 255];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("a", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int32 a(integer(8,false));
        }
    );
    assert_eq!(read_column::<u8>(&reader, "a"), values);
}

#[test]
fn parquet_array_i64() {
    let values = [-1_000_000_000_000_i64, 1, 2, 3, 1_000_000_000_000];
    let mut writer = PqArrayWriter::new(PqWriteOptions {
        row_group_size: 3,
        compression_level: 10,
        ..Default::default()
    });
    writer.add("x", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int64 x;
        }
    );
    assert_eq!(read_column::<i64>(&reader, "x"), values);
}

#[test]
fn parquet_array_nullable_f64() {
    let values = [Some(0.0_f64), Some(1.0), Some(2.0), None, None, Some(4.0)];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add_nullable("value", values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            optional double value;
        }
    );
    assert_eq!(read_nullable_column::<f64>(&reader, "value"), values);
}

#[test]
fn parquet_array_multi() {
    let values = [
        [0.0, 0.1, 0.2],
        [1.0, 1.1, 1.2],
        [2.0, 2.1, 2.2],
        [3.0, 3.1, 3.2],
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add_multiple(&["x", "y", "z"], values).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required double x;
            required double y;
            required double z;
        }
    );
    assert_eq!(
        read_multi_column::<f64, 3>(&reader, ["x", "y", "z"]),
        values
    );
}

#[test]
fn parquet_array_nullable_group() {
    let values = [
        Some([0.0_f32, 0.1]),
        None,
        Some([10.0, 0.2]),
        Some([20.0, 0.3]),
        Some([30.0, 0.4]),
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer
        .add_nullable_group("vectors", &["x", "y"], values)
        .unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            optional group vectors {
                required float x;
                required float y;
            }
        }
    );
    assert_eq!(
        read_nullable_group_column::<f32, 2>(&reader, "vectors", ["x", "y"]),
        values
    );
}

#[test]
fn parquet_array_separate_columns() {
    let values0 = [0_i64, 1, 2, 3, 4];
    let values1 = [Some(0.0_f64), Some(0.1), Some(0.2), Some(0.3), None];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("a", values0).unwrap();
    writer.add_nullable("b", values1).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int64 a;
            optional double b;
        }
    );
    assert_eq!(read_column::<i64>(&reader, "a"), values0);
    assert_eq!(read_nullable_column::<f64>(&reader, "b"), values1);
}

#[test]
fn parquet_array_separate_columns_uneven() {
    let values0 = [0_i64, 1, 2, 3, 4];
    let values1 = [0.0_f64, 0.1, 0.2, 0.3];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("a", values0).unwrap();
    writer.add("b", values1).unwrap();
    let mut buf = Vec::new();
    let e = writer.write(&mut buf).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Parquet error: uneven iterator lengths after 4 items"
    );
}

#[test]
fn parquet_array_string() {
    let values = [
        "foo".to_owned(),
        "bar".to_owned(),
        "".to_owned(),
        "♫".to_owned(),
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("s", values.clone()).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required byte_array s(string);
        }
    );
    assert_eq!(read_column::<String>(&reader, "s"), values);
}

#[test]
fn parquet_array_binary() {
    let values = [
        vec![0, 1, 2, 3],
        b"foo".to_vec(),
        b"".to_vec(),
        vec![254, 255],
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("b", values.clone()).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required byte_array b;
        }
    );
    assert_eq!(read_column::<Vec<u8>>(&reader, "b"), values);
}

#[test]
fn parquet_nullable_array_string() {
    let values = [
        Some("foo".to_owned()),
        Some("bar".to_owned()),
        None,
        Some("".to_owned()),
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer.add_nullable("a", values.clone()).unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            optional byte_array a(string);
        }
    );
    assert_eq!(read_nullable_column::<String>(&reader, "a"), values);
}

#[ignore = "for performance testing"]
#[test]
fn parquet_array_large() {
    let values: Vec<_> = (0..10_000_000).map(|i| f64::from(i)).collect();

    let mut writer = PqArrayWriter::new(Default::default());
    writer.add("value", values).unwrap();

    let mut start = std::time::Instant::now();
    let mut buf = Vec::new();
    writer.write(&mut buf).unwrap();
    println!("file size is {} B", buf.len());
    println!("wrote in {:?}", start.elapsed());

    start = std::time::Instant::now();
    let reader = PqArrayReader::new(Bytes::from(buf)).unwrap();
    let _read_column = read_column::<f64>(&reader, "value");
    println!("read in {:?}", start.elapsed());

    assert!(false);
}

#[test]
fn parquet_array_date() {
    let values = ["1970-01-01", "-1000-08-20", "2023-09-14", "+10000-12-31"];
    let mut writer = PqArrayWriter::new(Default::default());
    writer
        .add("d", values.iter().map(|s| NaiveDate::from_str(s).unwrap()))
        .unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int32 d (date);
        }
    );
    assert_eq!(
        reader
            .iter_column::<NaiveDate>("d")
            .unwrap()
            .map(|d| d.unwrap().to_string())
            .collect::<Vec<_>>(),
        values
    );
}

#[test]
fn parquet_array_date_time() {
    let values = [
        "1970-01-01 00:00:00 UTC",
        "-1000-08-20 03:00:00.123456 UTC",
        "2023-09-14 12:00:00 UTC",
        "+10000-12-31 23:59:59 UTC",
    ];
    let mut writer = PqArrayWriter::new(Default::default());
    writer
        .add(
            "t",
            values.iter().map(|s| DateTime::<Utc>::from_str(s).unwrap()),
        )
        .unwrap();
    let reader = write_read(writer);
    assert_eq!(
        reader.schema().as_ref(),
        &schema! {
            required int64 t (timestamp(micros, true));
        }
    );
    assert_eq!(
        reader
            .iter_column::<DateTime::<Utc>>("t")
            .unwrap()
            .map(|d| d.unwrap().to_string())
            .collect::<Vec<_>>(),
        values
    );
}
