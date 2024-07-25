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
    #[influxdb(field)]
    field2: i64,
    #[influxdb(field)]
    field3: String,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "something"]
struct Item2 {
    #[influxdb(tag)]
    name: String,
    #[influxdb(tag)]
    name2: String,
    #[influxdb(field)]
    field1: Option<u64>,
    #[influxdb(field)]
    field2: Option<i64>,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "barfoo"]
struct Item3 {
    #[influxdb(tag)]
    name: String,
    #[influxdb(field)]
    field1: Option<String>,
    #[influxdb(field)]
    field2: Option<String>,
    #[influxdb(field)]
    field3: Option<f64>,
    #[influxdb(field)]
    field4: Option<f64>,
    #[influxdb(field)]
    field5: Option<i64>,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "barfoo2"]
struct Item4 {
    #[influxdb(tag)]
    tag1: Option<String>,
    #[influxdb(tag)]
    tag2: Option<String>,
    #[influxdb(tag)]
    tag3: Option<String>,
    #[influxdb(tag)]
    tag4: Option<String>,
    #[influxdb(tag)]
    tag5: Option<String>,
    #[influxdb(field)]
    field1: String,
    #[influxdb(timestamp)]
    time: u64,
}


#[derive(WriteDataPoint)]
#[measurement = "foobar"]
struct Item5 {
    #[influxdb(tag)]
    tag1: Option<String>,
    #[influxdb(tag)]
    tag2: Option<String>,
    #[influxdb(tag)]
    tag3: Option<String>,
    #[influxdb(tag)]
    tag4: Option<String>,
    #[influxdb(tag)]
    tag5: Option<String>,
    #[influxdb(field)]
    field1: Option<String>,
    #[influxdb(field)]
    field2: Option<String>,
    #[influxdb(field)]
    field3: Option<f64>,
    #[influxdb(field)]
    field4: Option<f64>,
    #[influxdb(field)]
    field5: Option<i64>,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "allTagsNone"]
struct Item6 {
    #[influxdb(tag)]
    tag1: Option<String>,
    #[influxdb(tag)]
    tag2: Option<String>,
    #[influxdb(field)]
    field1: String,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "noTags"]
struct Item7 {
    #[influxdb(field)]
    field1: String,
    #[influxdb(timestamp)]
    time: u64,
}

#[derive(WriteDataPoint)]
#[measurement = "noTimestamp"]
struct Item8 {
    #[influxdb(tag)]
    tag1: String,
    #[influxdb(field)]
    field1: String,
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
        time: 222233u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"something,name=foo,name2=bar field1=32u,field2=33i,field3=\"hello\" 222233\n"
    );

    let item = Item2 {
        name: "foo".to_string(),
        name2: "bar".to_string(),
        field1: None,
        field2: Some(33i64),
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"something,name=foo,name2=bar field2=33i 222222\n"
    );

    let item = Item3 {
        name: "foo".to_string(),
        field1: None,
        field2: None,
        field3: Some(12.34),
        field4: None,
        field5: None,
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"barfoo,name=foo field3=12.34 222222\n"
    );

    let item = Item4 {
        tag1: None,
        tag2: None,
        tag3: Some("thisIsATag".to_string()),
        tag4: None,
        tag5: None,
        field1: "asdf".to_string(),
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"barfoo2,tag3=thisIsATag field1=\"asdf\" 222222\n"
    );

    let item = Item5 {
        tag1: None,
        tag2: None,
        tag3: Some("thisIsATag".to_string()),
        tag4: None,
        tag5: None,
        field1: None,
        field2: None,
        field3: Some(12.34),
        field4: None,
        field5: None,
        time: 222222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"foobar,tag3=thisIsATag field3=12.34 222222\n"
    );

    let item = Item6 {
    tag1: None,
    tag2: None,
    field1: "abc".to_string(),
    time: 122222u64
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"allTagsNone field1=\"abc\" 122222\n"
    );

    let item = Item7 {
        field1: "def".to_string(),
        time: 122222u64,
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"noTags field1=\"def\" 122222\n"
    );

    let item = Item8 {
        tag1: "abc".to_string(),
        field1: "def".to_string(),
    };

    let mut writer = Vec::new();
    item.write_data_point_to(&mut writer).unwrap();
    writer.flush().unwrap();
    println!("Writer: {}", std::str::from_utf8(&writer).unwrap());
    assert_eq!(
        &writer[..],
        b"noTimestamp,tag1=abc field1=\"def\"\n"
    );
}
