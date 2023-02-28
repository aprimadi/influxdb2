use influxdb2_derive::WriteDataPoint;

#[derive(WriteDataPoint)]
#[measurement = "something"]
struct Item {
    #[influxdb(tag)]
    name: String,
    #[influxdb(tag)]
    name2: String,
    #[influxdb(field)]
    field1: u64,
    field2: i64,
    field3: String,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "something"]
struct Item2 {
    #[influxdb(tag)]
    name: Option<String>,
    #[influxdb(tag)]
    name2: Option<String>,
    #[influxdb(field)]
    field1: Option<u64>,
    field2: i64,
    #[influxdb(timestamp)]
    time: u64,
}
fn main() {
    use influxdb2::models::WriteDataPoint;
    use std::io::Write;
    let item = Item {
        name: "foo".to_string(),
        name2: "bar".to_string(),
        field1: 32u64,
        field2: 33i64,
        field3: "hello".to_string(),
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("{}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"something,name=foo,name2=bar field1=32u,field2=33i,field3=\"hello\" 222222\n"
    );

    let item = Item2 {
        name: Some("foo".to_string()),
        name2: None,
        field1: None,
        field2: 33i64,
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("{}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"something,name=foo,name2=None field1=\"None\",field2=33i 222222\n"
    )
}
