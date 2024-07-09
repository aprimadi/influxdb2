//! Query
//!
//! Query InfluxDB using InfluxQL or Flux Query

use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;

use crate::{Client, Http, RequestError, ReqwestProcessing, Serializing};

use base64::decode;
use chrono::DateTime;
use csv::StringRecord;
use fallible_iterator::FallibleIterator;
use go_parse_duration::parse_duration;
use influxdb2_structmap::value::Value;
use influxdb2_structmap::{FromMap, GenericMap};
use ordered_float::OrderedFloat;
use reqwest::{Method, StatusCode};
use snafu::ResultExt;

use crate::models::{
    AnalyzeQueryResponse, AstResponse, FluxSuggestion, FluxSuggestions, LanguageRequest, Query,
};

/// Allows for multiple iterators over the result
#[derive(Clone, Debug)]
pub struct QueryTableIter {
    text: String,
}

impl<'a> QueryTableIter {
    fn new(text: String) -> Self {
        Self { text }
    }

    /// Get the iterator
    pub fn result(&'a self) -> impl FallibleIterator<Item = FluxRecord, Error = RequestError> + 'a {
        QueryTableResult::new(&self.text)
    }

    /// Is the response empty?
    pub fn is_empty(&self) -> bool {
        matches!(QueryTableResult::new(&self.text).next(), Ok(None))
    }
}

impl Client {
    /// Get Query Suggestions
    pub async fn query_suggestions(&self) -> Result<FluxSuggestions, RequestError> {
        let req_url = self.url("/api/v2/query/suggestions");
        let response = self
            .request(Method::GET, &req_url)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<FluxSuggestions>()
                .await
                .context(ReqwestProcessing)?),
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Query Suggestions with name
    pub async fn query_suggestions_name(&self, name: &str) -> Result<FluxSuggestion, RequestError> {
        let req_url = self.url(&format!(
            "/api/v2/query/suggestions/{name}",
            name = crate::common::urlencode(name)
        ));

        let response = self
            .request(Method::GET, &req_url)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<FluxSuggestion>()
                .await
                .context(ReqwestProcessing)?),
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Query
    pub async fn query<T: FromMap>(&self, query: Option<Query>) -> Result<Vec<T>, RequestError> {
        let req_url = self.url("/api/v2/query");
        let body = serde_json::to_string(&query.unwrap_or_default()).context(Serializing)?;

        let response = self
            .request(Method::POST, &req_url)
            .header("Accepting-Encoding", "identity")
            .header("Content-Type", "application/json")
            .query(&[("org", &self.org)])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => {
                let text = response.text().await.unwrap();
                let qtr = QueryTableResult::new(&text[..]);
                let qr = QueryResult::new(qtr)?;
                let mut res = vec![];
                for item in qr.items {
                    res.push(T::from_genericmap(item));
                }
                Ok(res)
            }
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Query Raw
    pub async fn query_raw(&self, query: Option<Query>) -> Result<Vec<FluxRecord>, RequestError> {
        let req_url = self.url("/api/v2/query");
        let body = serde_json::to_string(&query.unwrap_or_default()).context(Serializing)?;

        let response = self
            .request(Method::POST, &req_url)
            .header("Accepting-Encoding", "identity")
            .header("Content-Type", "application/json")
            .query(&[("org", &self.org)])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => {
                let text = response.text().await.unwrap();
                let qtr = QueryTableResult::new(&text[..]);
                let mut records = vec![];
                for record in qtr.iterator() {
                    records.push(record?);
                }
                Ok(records)
            }
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Query return an iterator over the raw results
    /// Saves on memory usage
    pub async fn query_raw_iter(
        &self,
        query: Option<Query>,
    ) -> Result<QueryTableIter, RequestError> {
        let req_url = self.url("/api/v2/query");
        let body = serde_json::to_string(&query.unwrap_or_default()).context(Serializing)?;

        let response = self
            .request(Method::POST, &req_url)
            .header("Accepting-Encoding", "identity")
            .header("Content-Type", "application/json")
            .query(&[("org", &self.org)])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => {
                let text = response.text().await.context(ReqwestProcessing)?;

                Ok(QueryTableIter::new(text))
            }
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Analyze Query
    pub async fn query_analyze(
        &self,
        query: Option<Query>,
    ) -> Result<AnalyzeQueryResponse, RequestError> {
        let req_url = self.url("/api/v2/query/analyze");

        let response = self
            .request(Method::POST, &req_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&query.unwrap_or_default()).context(Serializing)?)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<AnalyzeQueryResponse>()
                .await
                .context(ReqwestProcessing)?),
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Get Query AST Response
    pub async fn query_ast(
        &self,
        language_request: Option<LanguageRequest>,
    ) -> Result<AstResponse, RequestError> {
        let req_url = self.url("/api/v2/query/ast");

        let response = self
            .request(Method::POST, &req_url)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&language_request.unwrap_or_default())
                    .context(Serializing)?,
            )
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => Ok(response
                .json::<AstResponse>()
                .await
                .context(ReqwestProcessing)?),
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }

    /// Returns bucket measurements
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket name
    /// * `start` - Optional start time. Default is `-30d`
    /// * `stop` - Optional stop time. Default is `now()`
    pub async fn list_measurements(
        &self,
        bucket: &str,
        start: Option<&str>,
        stop: Option<&str>,
    ) -> Result<Vec<String>, RequestError> {
        let mut params = vec![];
        params.push(format!(r#"bucket: "{bucket}""#));
        if let Some(start) = start {
            params.push(format!("start: {start}"));
        }
        if let Some(stop) = stop {
            params.push(format!("stop: {stop}"));
        }
        let params = params.join(", ");

        let query = Query::new(format!(
            r#"import "influxdata/influxdb/schema"

            schema.measurements({params})"#
        ));
        self.exec_schema_query(query).await
    }

    /// List field keys for measurement
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket name
    /// * `measurement` - The measurement name
    /// * `start` - Optional start time. Default is `-30d`
    /// * `stop` - Optional stop time. Default is `now()`
    pub async fn list_measurement_field_keys(
        &self,
        bucket: &str,
        measurement: &str,
        start: Option<&str>,
        stop: Option<&str>,
    ) -> Result<Vec<String>, RequestError> {
        let mut params = vec![];
        params.push(format!(r#"bucket: "{bucket}""#));
        params.push(format!(r#"measurement: "{measurement}""#));
        if let Some(start) = start {
            params.push(format!("start: {start}"));
        }
        if let Some(stop) = stop {
            params.push(format!("stop: {stop}"));
        }
        let params = params.join(", ");

        let query = Query::new(format!(
            r#"import "influxdata/influxdb/schema"

            schema.measurementFieldKeys({params})"#,
        ));
        self.exec_schema_query(query).await
    }

    /// List all tag values for measurement tag
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket name
    /// * `measurement` - The measurement name
    /// * `tag` - The tag name
    /// * `start` - Optional start time. Default is `-30d`
    /// * `stop` - Optional stop time. Default is `now()`
    pub async fn list_measurement_tag_values(
        &self,
        bucket: &str,
        measurement: &str,
        tag: &str,
        start: Option<&str>,
        stop: Option<&str>,
    ) -> Result<Vec<String>, RequestError> {
        let mut params = vec![];
        params.push(format!(r#"bucket: "{bucket}""#));
        params.push(format!(r#"measurement: "{measurement}""#));
        params.push(format!(r#"tag: "{tag}""#));
        if let Some(start) = start {
            params.push(format!("start: {start}"));
        }
        if let Some(stop) = stop {
            params.push(format!("stop: {stop}"));
        }
        let params = params.join(", ");

        let query = Query::new(format!(
            r#"import "influxdata/influxdb/schema"

            schema.measurementTagValues({params})"#,
        ));
        self.exec_schema_query(query).await
    }

    /// List all tag keys for measurement
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket name
    /// * `measurement` - The measurement name
    /// * `start` - Optional start time. Default is `-30d`
    /// * `stop` - Optional stop time. Default is `now()`
    pub async fn list_measurement_tag_keys(
        &self,
        bucket: &str,
        measurement: &str,
        start: Option<&str>,
        stop: Option<&str>,
    ) -> Result<Vec<String>, RequestError> {
        let mut params = vec![];
        params.push(format!(r#"bucket: "{bucket}""#));
        params.push(format!(r#"measurement: "{measurement}""#));
        if let Some(start) = start {
            params.push(format!("start: {start}"));
        }
        if let Some(stop) = stop {
            params.push(format!("stop: {stop}"));
        }
        let params = params.join(", ");

        let query = Query::new(format!(
            r#"import "influxdata/influxdb/schema"

            schema.measurementTagKeys({params})"#,
        ));
        self.exec_schema_query(query).await
    }

    async fn exec_schema_query(&self, query: Query) -> Result<Vec<String>, RequestError> {
        let req_url = self.url("/api/v2/query");
        let body = serde_json::to_string(&query).context(Serializing)?;

        let response = self
            .request(Method::POST, &req_url)
            .header("Accepting-Encoding", "identity")
            .header("Content-Type", "application/json")
            .query(&[("org", &self.org)])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;

        match response.status() {
            StatusCode::OK => {
                let text: String = response.text().await.unwrap();

                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .comment(Some(b'#'))
                    .from_reader(text.as_bytes());

                Ok(reader
                    .records()
                    .flatten()
                    .flat_map(|r: StringRecord| r.get(3).map(|s| s.to_owned()))
                    .collect())
            }
            status => {
                let text = response.text().await.context(ReqwestProcessing)?;
                Http { status, text }.fail()?
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DataType {
    String,
    Double,
    Bool,
    Long,
    UnsignedLong,
    Duration,
    Base64Binary,
    TimeRFC,
}

impl FromStr for DataType {
    type Err = RequestError;

    fn from_str(input: &str) -> Result<Self, RequestError> {
        match input {
            "string" => Ok(Self::String),
            "double" => Ok(Self::Double),
            "boolean" => Ok(Self::Bool),
            "long" => Ok(Self::Long),
            "unsignedLong" => Ok(Self::UnsignedLong),
            "duration" => Ok(Self::Duration),
            "base64Binary" => Ok(Self::Base64Binary),
            "dateTime:RFC3339" => Ok(Self::TimeRFC),
            "dateTime:RFC3339Nano" => Ok(Self::TimeRFC),
            _ => Err(RequestError::Deserializing {
                text: format!("unknown datatype: {}", input),
            }),
        }
    }
}

struct FluxColumn {
    name: String,
    data_type: DataType,
    group: bool,
    default_value: String,
}

/// Represents a flux record returned from a query.
#[derive(Clone, Debug, PartialEq)]
pub struct FluxRecord {
    /// Table id
    pub table: i32,
    /// Map of key/value pairs
    pub values: GenericMap,
}

struct FluxTableMetadata {
    position: i32,
    columns: Vec<FluxColumn>,
}

struct QueryTableResult<'a> {
    csv_reader: csv::Reader<&'a [u8]>,
    table_position: i32,
    table_changed: bool,
    table: Option<FluxTableMetadata>,
}

#[derive(PartialEq)]
enum ParsingState {
    Normal,
    Annotation,
    Error,
}

impl<'a> QueryTableResult<'a> {
    fn new(text: &'a str) -> Self {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());
        Self {
            csv_reader: reader,
            table_position: 0,
            table_changed: false,
            table: None,
        }
    }
}

impl<'a> FallibleIterator for QueryTableResult<'a> {
    type Item = FluxRecord;
    type Error = RequestError;

    fn next(&mut self) -> Result<Option<FluxRecord>, RequestError> {
        // Hold the FluxRecord to be returned.
        let record: FluxRecord;

        self.table_changed = false;
        let mut row = StringRecord::new();
        let mut parsing_state = ParsingState::Normal;
        let mut data_type_annotation_found = false;
        loop {
            if !self.csv_reader.read_record(&mut row).unwrap() {
                // EOF
                return Ok(None);
            }
            if row.len() <= 1 {
                continue;
            }
            if let Some(s) = row.get(0) {
                if !s.is_empty() && s.chars().nth(0).unwrap() == '#' {
                    // Finding new table, prepare for annotation parsing
                    if parsing_state == ParsingState::Normal {
                        self.table = Some(FluxTableMetadata {
                            position: self.table_position,
                            columns: Vec::new(),
                        });
                        self.table_position += 1;
                        self.table_changed = true;
                        for _ in 1..row.len() {
                            self.table.as_mut().unwrap().columns.push(FluxColumn {
                                name: String::from(""),
                                data_type: DataType::String,
                                group: false,
                                default_value: String::from(""),
                            });
                        }
                        parsing_state = ParsingState::Annotation;
                    }
                }
            }
            if self.table.is_none() {
                return Err(RequestError::Deserializing {
                    text: String::from("annotations not found"),
                });
            }
            if row.len() - 1 != self.table.as_ref().unwrap().columns.len() {
                return Err(RequestError::Deserializing {
                    text: format!(
                        "row has different number of columns than the table: {} vs {}",
                        row.len() - 1,
                        self.table.as_ref().unwrap().columns.len(),
                    ),
                });
            }
            if let Some(s) = row.get(0) {
                match s {
                    "" => {
                        match parsing_state {
                            ParsingState::Annotation => {
                                // Parse column name (csv header)
                                if !data_type_annotation_found {
                                    return Err(RequestError::Deserializing {
                                        text: String::from("datatype annotation not found"),
                                    });
                                }
                                if row.get(1).unwrap() == "error" {
                                    parsing_state = ParsingState::Error;
                                } else {
                                    for i in 1..row.len() {
                                        let column =
                                            &mut self.table.as_mut().unwrap().columns[i - 1];
                                        column.name = String::from(row.get(i).unwrap());
                                    }
                                    parsing_state = ParsingState::Normal;
                                }
                                continue;
                            }
                            ParsingState::Error => {
                                let msg = if row.len() > 1 && !row.get(1).unwrap().is_empty() {
                                    row.get(1).unwrap()
                                } else {
                                    "unknown query error"
                                };
                                let mut reference = String::from("");
                                if row.len() > 2 && !row.get(2).unwrap().is_empty() {
                                    let s = row.get(2).unwrap();
                                    reference = format!(",{}", s);
                                }
                                return Err(RequestError::Deserializing {
                                    text: format!("{}{}", msg, reference),
                                });
                            }
                            _ => {}
                        }
                        let mut values = BTreeMap::new();
                        for i in 1..row.len() {
                            let column = &self.table.as_mut().unwrap().columns[i - 1];
                            let mut v = row.get(i).unwrap();
                            if v.is_empty() {
                                v = &column.default_value[..];
                            }
                            let value = parse_value(v, column.data_type, column.name.as_str())?;
                            values.entry(column.name.clone()).or_insert(value);
                        }
                        record = FluxRecord {
                            table: self.table.as_ref().unwrap().position,
                            values,
                        };
                        break;
                    }
                    "#datatype" => {
                        data_type_annotation_found = true;
                        for i in 1..row.len() {
                            let column = &mut self.table.as_mut().unwrap().columns[i - 1];
                            let dt = DataType::from_str(row.get(i).unwrap())?;
                            column.data_type = dt;
                        }
                    }
                    "#group" => {
                        for i in 1..row.len() {
                            let column = &mut self.table.as_mut().unwrap().columns[i - 1];
                            column.group = row.get(i).unwrap() == "true";
                        }
                    }
                    "#default" => {
                        for i in 1..row.len() {
                            let column = &mut self.table.as_mut().unwrap().columns[i - 1];
                            column.default_value = String::from(row.get(i).unwrap());
                        }
                    }
                    _ => {
                        return Err(RequestError::Deserializing {
                            text: format!("invalid first cell: {}", s),
                        });
                    }
                }
            }
        }
        Ok(Some(record))
    }
}

struct QueryResult {
    items: Vec<GenericMap>,
}

impl QueryResult {
    fn new(qtr: QueryTableResult<'_>) -> Result<Self, RequestError> {
        let ignored_keys = vec!["_field", "_value", "table"];
        let ignored_keys: HashSet<&str> = ignored_keys.into_iter().collect();

        // Construct build table, this groups values with the same tags and
        // timestamp but in different table.
        //
        // We need to do this because influxdb v2 stores multiple fields in
        // different tables even though it's part of the same measurement.
        let mut build_table = HashMap::<GenericMap, GenericMap>::new();
        let mut key_order: Vec<GenericMap> = vec![];
        for record in qtr.iterator() {
            let mut record_values = record?.values;

            // Construct key
            let mut key = record_values.clone();
            key.retain(|k, _| !ignored_keys.contains(k.as_str()));

            match build_table.get_mut(&key) {
                Some(entry) => {
                    // Set field value
                    let field;
                    if let Value::String(f) = record_values.get("_field").unwrap() {
                        field = f.clone();
                    } else {
                        unreachable!();
                    }
                    let value = record_values.get("_value").unwrap();
                    entry.insert(field, value.clone());
                }
                None => {
                    // Set field value
                    let field;
                    if let Value::String(f) = record_values.get("_field").unwrap() {
                        field = f.clone();
                    } else {
                        unreachable!();
                    }
                    let value = record_values.get("_value").unwrap();
                    record_values.insert(field, value.clone());

                    build_table.insert(key.clone(), record_values);
                    key_order.push(key);
                }
            }
        }

        // Build items based on the order the `key` is inserted
        let mut items = vec![];
        for key in key_order {
            let entry = build_table.get(&key).unwrap();
            items.push(entry.clone());
        }

        Ok(Self { items })
    }
}

fn parse_value(s: &str, t: DataType, name: &str) -> Result<Value, RequestError> {
    match t {
        DataType::String => Ok(Value::String(String::from(s))),
        DataType::Double => {
            let v = s.parse::<f64>().unwrap();
            Ok(Value::Double(OrderedFloat::from(v)))
        }
        DataType::Bool => {
            if s.to_lowercase() == "false" {
                Ok(Value::Bool(false))
            } else {
                Ok(Value::Bool(true))
            }
        }
        DataType::Long => {
            let v = s.parse::<i64>().unwrap();
            Ok(Value::Long(v))
        }
        DataType::UnsignedLong => {
            let v = s.parse::<u64>().unwrap();
            Ok(Value::UnsignedLong(v))
        }
        DataType::Duration => match parse_duration(s) {
            Ok(d) => Ok(Value::Duration(chrono::Duration::nanoseconds(d))),
            Err(_) => Err(RequestError::Deserializing {
                text: format!("invalid duration: {}, name: {}", s, name),
            }),
        },
        DataType::Base64Binary => {
            let b = decode(s).unwrap();
            Ok(Value::Base64Binary(b))
        }
        DataType::TimeRFC => {
            let t = DateTime::parse_from_rfc3339(s).unwrap();
            Ok(Value::TimeRFC(t))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FromDataPoint;
    use mockito::{mock, Matcher};

    #[derive(Default, FromDataPoint)]
    struct Empty {}

    #[tokio::test]
    async fn query_suggestions() {
        let token = "some-token";

        let mock_server = mock("GET", "/api/v2/query/suggestions")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_suggestions().await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_suggestions_name() {
        let token = "some-token";
        let suggestion_name = "some-name";

        let mock_server = mock(
            "GET",
            format!(
                "/api/v2/query/suggestions/{name}",
                name = crate::common::urlencode(suggestion_name)
            )
            .as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_suggestions_name(suggestion_name).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query() {
        let token = "some-token";
        let org = "some-org";
        let query: Option<Query> = Some(Query::new("some-influx-query-string".to_string()));
        let mock_server = mock("POST", "/api/v2/query")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Accepting-Encoding", "identity")
            .match_header("Content-Type", "application/json")
            .match_query(Matcher::UrlEncoded("org".into(), org.into()))
            .match_body(
                serde_json::to_string(&query.clone().unwrap_or_default())
                    .unwrap()
                    .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), org, token);

        let _result = client.query::<Empty>(query).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_opt() {
        let token = "some-token";
        let org = "some-org";

        let mock_server = mock("POST", "/api/v2/query")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Accepting-Encoding", "identity")
            .match_header("Content-Type", "application/json")
            .match_query(Matcher::UrlEncoded("org".into(), org.into()))
            .match_body(serde_json::to_string(&Query::default()).unwrap().as_str())
            .create();

        let client = Client::new(mockito::server_url(), org, token);

        let _result = client.query::<Empty>(None).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_analyze() {
        let token = "some-token";
        let query: Option<Query> = Some(Query::new("some-influx-query-string".to_string()));
        let mock_server = mock("POST", "/api/v2/query/analyze")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(
                serde_json::to_string(&query.clone().unwrap_or_default())
                    .unwrap()
                    .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_analyze(query).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_analyze_opt() {
        let token = "some-token";
        let query: Option<Query> = None;
        let mock_server = mock("POST", "/api/v2/query/analyze")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(
                serde_json::to_string(&query.clone().unwrap_or_default())
                    .unwrap()
                    .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_analyze(query).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_ast() {
        let token = "some-token";
        let language_request: Option<LanguageRequest> =
            Some(LanguageRequest::new("some-influx-query-string".to_string()));
        let mock_server = mock("POST", "/api/v2/query/ast")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(
                serde_json::to_string(&language_request.clone().unwrap_or_default())
                    .unwrap()
                    .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_ast(language_request).await;

        mock_server.assert();
    }

    #[tokio::test]
    async fn query_ast_opt() {
        let token = "some-token";
        let language_request: Option<LanguageRequest> = None;
        let mock_server = mock("POST", "/api/v2/query/ast")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(
                serde_json::to_string(&language_request.clone().unwrap_or_default())
                    .unwrap()
                    .as_str(),
            )
            .create();

        let client = Client::new(mockito::server_url(), "org", token);

        let _result = client.query_ast(language_request).await;

        mock_server.assert();
    }

    #[test]
    fn test_query_table_result() {
        let text = "#datatype,string,long,dateTime:RFC3339,dateTime:RFC3339,dateTime:RFC3339,double,string,string,string,string
#group,false,false,true,true,false,false,true,true,true,true
#default,_result,,,,,,,,,
,result,table,_start,_stop,_time,_value,_field,_measurement,a,b
,,0,2020-02-17T22:19:49.747562847Z,2020-02-18T22:19:49.747562847Z,2020-02-18T10:34:08.135814545Z,1.4,f,test,1,adsfasdf
,,0,2020-02-17T22:19:49.747562847Z,2020-02-18T22:19:49.747562847Z,2020-02-18T22:08:44.850214724Z,6.6,f,test,1,adsfasdf
";
        let qtr = QueryTableResult::new(text);
        #[rustfmt::skip]
        let expected: [FluxRecord; 2] = [
            FluxRecord {
                table: 0,
                values: [
                    (String::from("result"), Value::String(String::from("_result"))),
                    (String::from("table"), Value::Long(0)),
                    (String::from("_start"), parse_value("2020-02-17T22:19:49.747562847Z", DataType::TimeRFC, "_start").unwrap()),
                    (String::from("_stop"), parse_value("2020-02-18T22:19:49.747562847Z", DataType::TimeRFC, "_stop").unwrap()),
                    (String::from("_time"), parse_value("2020-02-18T10:34:08.135814545Z", DataType::TimeRFC, "_time").unwrap()),
                    (String::from("_field"), Value::String(String::from("f"))),
                    (String::from("_measurement"), Value::String(String::from("test"))),
                    (String::from("_value"), Value::Double(OrderedFloat::from(1.4))),
                    (String::from("a"), Value::String(String::from("1"))),
                    (String::from("b"), Value::String(String::from("adsfasdf"))),
                ].iter().cloned().collect(),
            },
            FluxRecord {
                table: 0,
                values: [
                    (String::from("result"), Value::String(String::from("_result"))),
                    (String::from("table"), Value::Long(0)),
                    (String::from("_start"), parse_value("2020-02-17T22:19:49.747562847Z", DataType::TimeRFC, "_start").unwrap()),
                    (String::from("_stop"), parse_value("2020-02-18T22:19:49.747562847Z", DataType::TimeRFC, "_stop").unwrap()),
                    (String::from("_time"), parse_value("2020-02-18T22:08:44.850214724Z", DataType::TimeRFC, "_time").unwrap()),
                    (String::from("_field"), Value::String(String::from("f"))),
                    (String::from("_measurement"), Value::String(String::from("test"))),
                    (String::from("_value"), Value::Double(OrderedFloat::from(6.6))),
                    (String::from("a"), Value::String(String::from("1"))),
                    (String::from("b"), Value::String(String::from("adsfasdf"))),
                ].iter().cloned().collect(),
            },
        ];
        for (i, item) in qtr.iterator().enumerate() {
            match item {
                Ok(record) => {
                    assert_eq!(record, expected[i]);
                }
                Err(e) => {
                    assert_eq!(format!("{}", e), "");
                }
            }
        }
    }
}
