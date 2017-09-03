use chrono;
use num;
use serde;
use std;
use super::Error;
use super::Result;

use serde::de::IntoDeserializer;

/// The default is the decimal separator of the system.
/// 
/// Symbol used to separate the integral part of a number from the fractional part.
/// A decimal separator usually is a dot or a comma.
#[derive(Debug, Deserialize, Serialize)]
pub enum DecimalSeparator {
  #[serde(rename=".")]
  /// Dot character, ASCII \0x2E
  Dot,

  /// Comma character, ASCII \0x2C
  #[serde(rename=",")]
  Comma,
}

///  Specifies which x-values are saved.
#[derive(Debug, Deserialize, Serialize)]
pub enum XColumns {
  /// Save no x-values.
  /// 
  /// The first data column is blank.
  /// The x-values can be generated from the X0 and Delta_X values.
  No,

  /// Saves one column of x-values.
  /// This column corresponds to the first column of data that contains the most number of samples.
  One,

  /// Saves a column of x data for every column of y data.
  Multi,
}

impl Default for XColumns {
  fn default() -> Self { XColumns::One }
}

type OperatorName = String;
type ProjectName = String;
type TestNumber = String;
type DataRow = (Vec<f64>, Option<String>);

#[derive(Debug, Deserialize, Serialize)]
pub struct Measurement {
  pub header: MeasurementHeader,
  pub data_headings: Vec<String>,
  pub data: Vec<DataRow>,
}

/// Date which (de)serializes to string form YYYY/MM/DD
#[derive(Debug)]
pub struct Date(chrono::NaiveDate);

impl std::str::FromStr for Date {
  type Err = chrono::format::ParseError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(Date(chrono::NaiveDate::parse_from_str(s, "%Y/%m/%d")?))
  }
}
impl<'de> serde::de::Deserialize<'de> for Date {
  fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> std::result::Result<Date, D::Error> {
    deserializer.deserialize_str(DateVisitor)
  }
}
impl std::fmt::Display for Date {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    self.0.format("%Y/%m/%d").fmt(f)
  }
}

impl serde::ser::Serialize for Date {
  fn serialize<S: serde::ser::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(self.to_string().as_ref())
  }
}

struct DateVisitor;

impl<'de> serde::de::Visitor<'de> for DateVisitor {
  type Value = Date;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("A date in the format YYYY/MM/DD")
  }

  fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
    use std::str::FromStr;
    Self::Value::from_str(value).map_err(serde::de::Error::custom)
  }
}

/// LVM File
#[derive(Debug, Deserialize, Serialize)]
pub struct File {
  pub header: FileHeader,
  pub measurements: Vec<Measurement>,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct MeasurementHeader {
  /// Number of channels in the packet.
  ///
  /// This field must occur before any fields that depend on it.
  /// For example, the Samples field has entries for each channel,
  /// so the reader must know the number of channels to properly parse it.
  #[serde(rename="Channels")]
  pub channels: (usize, Vec<String>),

  /// Date the data set in the segment started.
  /// 
  /// There are separate dates for each data set.
  /// The dates are placed in the same column as the y data of the data set.
  #[serde(rename="Date")]
  pub date: Vec<Date>,

  /// The increment between points on the x-axis.
  /// 
  /// The .lvm format assumes all data is equally spaced in the x-dimension.
  /// There is one value for each data set in the packet.
  /// The value appears in the same column as the y-values for the data.
  #[serde(rename="Delta_X")]
  pub delta_x: Vec<f32>,

  /// Comments the user adds to the segment header.
  /// 
  /// A segment header does not necessarily exist for every packet.
  /// Use the Comment field at the far right of the data for specific notes about each segment.
  #[serde(rename="Notes")]
  pub notes: Option<String>,

  /// Number of samples in each waveform in the packet.
  #[serde(rename="Samples")]
  pub samples: Vec<usize>,

  /// Name of the test that acquired the segment of data.
  #[serde(rename="Test_Name")]
  pub test_name: Option<String>,

  /// Test numbers in the Test_Series that acquired the data in this segment.
  #[serde(rename="Test_Number")]
  pub test_numbers: Option<TestNumber>,

  /// Series of the test performed to get the data in this packet.
  #[serde(rename="Test_Series")]
  pub test_series: Option<String>,

  /// Time of day when you started acquiring the data set in the segment.
  ///
  /// Each data set includes a different time.
  /// The times are placed in the same column as the y data of the data set.
  #[serde(rename="Time")]
  pub time: Vec<Time>,

  /// Model number of the unit under test.
  #[serde(rename="UUT_M/N")]
  pub uut_mn: Option<String>,

  /// Name or instrument class of the unit under test.
  #[serde(rename="UUT_Name")]
  pub uut_name: Option<String>,

  /// Serial number of the unit under test.
  #[serde(rename="UUT_S/N")]
  pub uut_sn: Option<String>,

  /// The initial value for the x-axis.
  ///
  /// Each data set in the packet has a single X0 value.
  /// The value appears in the same column as the y-values for the data.
  #[serde(rename="X0")]
  pub x0: Vec<f32>,

  /// Unit type of the x-axis.
  ///
  /// The actual data does not need to be in SI units.
  /// The ```X_Unit_Label``` field indicates the actual units of the data.
  #[serde(rename="X_Dimension")]
  pub x_dimension: Option<Vec<UnitType>>,

  /// Labels for the units used in plotting the x data.
  /// 
  /// The label appears in the same column as the y data to which it corresponds.
  /// You do not have to fill in all unit labels.
  #[serde(rename="X_Unit_Label")]
  pub x_unit_label: Option<Vec<Unit>>,

  /// Unit type of the y-axis.
  /// 
  /// The actual data does not need to be in SI units.
  /// The ```Y_Unit_Label``` field indicates the actual units of the data.
  #[serde(default,rename="Y_Dimension")]
  pub y_dimension: UnitType,

  #[serde(rename="Y_Unit_Label")]
  pub y_unit_label: Vec<Unit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileHeader {
  /// Date when the data collection started.
  #[serde(rename="Date")]
  pub date: Date,

  /// Date when the data collection started.
  #[serde(rename="Description")]
  pub description: Option<String>,

  /// The default is the decimal separator of the system.
  /// 
  /// Symbol used to separate the integral part of a number from the fractional part.
  /// A decimal separator usually is a dot or a comma.
  /// 
  /// required for version 2.0.
  #[serde(rename="Decimal_Separator")]
  pub decimal_separator: DecimalSeparator,

  /// Specifies whether each packet has a header.
  #[serde(default, rename="Multi_Headings")]
  pub multi_headings: bool,

  /// Operator who generated these measurements
  #[serde(rename="Operator")]
  pub operator: Option<OperatorName>,

  /// Name of the project associated with the data in the file.
  #[serde(rename="Project")]
  pub project: Option<ProjectName>,

  /// Version number of reader needed to parse the file correctly the file type.
  /// 
  /// For example, a 1.0 version of a reader can parse the file until the file format changes
  /// so much that it is no longer backwards compatible.
  /// The ```Writer_Version``` supplies the actual file type version.
  #[serde(rename="Reader_Version")]
  pub reader_version: Version,

  /// Character(s) used to separate each field in the file.
  /// 
  /// You can use any character as a separator except the new line character.
  /// However, base-level readers usually use tabs or commas as delimiters.
  /// Escape other text fields in the file to prevent the separator character(s)
  /// from appearing as text rather than delimiters.
  /// To use the character(s) specified as a delimiter requires
  /// that you know what that character(s) is.
  /// To find out what the separator character(s) is,
  /// read the entire header block and search for the keyword Separator.
  /// The character(s) that follows the keyword is the separator.
  /// To parse the file faster, place this field in the header
  /// after the LabVIEW Measurement ID field.
  /// To read in the entire header block, read until you find the ***End_of_Header*** tag.
  #[serde(default, rename="Separator")]
  pub separator: Separator,

  /// Time at which the start of a data series occurred.
  #[serde(rename="Time")]
  pub time: Time,

  /// Format of the x-axis values.
  /// 
  /// This tag is valid only if the ```X_Dimension``` tag value is set to Time.
  #[serde(rename="Time_Pref")]
  pub time_pref: TimePref,

  /// Version number of the file type written by the software.
  #[serde(rename="Writer_Version")]
  pub writer_version: Version,

  ///  Specifies which x-values are saved.
  #[serde(default, rename="X_Columns")]
  pub x_columns: XColumns,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Separator {
  Comma,
  Tab,
  Other(char),
}

impl Default for Separator {
  fn default() -> Self { Separator::Tab }
}

#[derive(Debug)]
pub struct Time(chrono::NaiveTime);

impl std::fmt::Display for Time {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    self.0.format("%H:%M:%S%.f").fmt(f)
  }
}

impl std::str::FromStr for Time {
  type Err = chrono::format::ParseError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(Time(chrono::NaiveTime::parse_from_str(s, "%H:%M:%S%.f")?))
  }
}

impl<'de> serde::de::Deserialize<'de> for Time {
  fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> std::result::Result<Time, D::Error> {
    deserializer.deserialize_str(TimeVisitor)
  }
}

impl serde::ser::Serialize for Time {
  fn serialize<S: serde::ser::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(self.to_string().as_ref())
  }
}

struct TimeVisitor;

impl<'de> serde::de::Visitor<'de> for TimeVisitor {
  type Value = Time;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("A time in the format HH:MM:SS.XXX")
  }

  fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
    use std::str::FromStr;
    Self::Value::from_str(value).map_err(serde::de::Error::custom)
  }
}


#[derive(Debug, Deserialize, Serialize)]
pub enum TimePref {
  /// x-value is number of seconds since midnight, January 1, 1904 GMT
  Absolute,

  /// x-value is number of seconds since the date and time stamps
  Relative,
}

impl Default for TimePref {
  fn default() -> Self { TimePref::Relative }
}


#[derive(Debug, Deserialize, Serialize)]
pub enum Unit {
  Milliamps,
  Volts,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UnitType {
  /// Electric Potential (Joulse)
  #[serde(rename="Electric_Potential")]
  ElectricPotential,

  /// Time (seconds)
  Time,
}

impl Default for UnitType {
  fn default() -> Self { UnitType::ElectricPotential }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Version(u8);

struct Deserializer<R: std::io::BufRead> {
  line_current: String,
  line_index: usize,
  input: std::io::Lines<R>,
  input_eof: bool,
  separator: char,
  sequence_style: SequenceStyle,
}

const HEADER: &'static str = "LabVIEW Measurement";

impl<R: std::io::BufRead> Deserializer<R> {
  const SEPARATOR: char = '\t';

  #[must_use]
  fn from_reader(i_reader: R) -> Result<Self> {
    let mut d = Deserializer {
      input: i_reader.lines(),
      input_eof: false,
      line_current: String::new(),
      line_index: 0,
      separator: Self::SEPARATOR,
      sequence_style: SequenceStyle::Following,
    };
    d.parse_newline()?;

    match d.parse_token()?.as_ref() {
      HEADER => {
        d.separator = char::from(d.line_current.pop().ok_or(Error::UnexpectedEol)?);
        d.parse_newline()?;
        Ok(d)
      },
      //FIXME: More efficient string storing
      h => Err(Error::UnexpectedToken(h.to_string())),
    }
  }

  #[must_use]
  fn peek_char(&mut self) -> Option<char> {
    if self.input_eof {
      None
    } else {
      self.line_current.chars().next()
    }
  }

  #[must_use]
  fn peek_newline(&mut self) -> bool {
    self.line_current.is_empty()
  }

  #[must_use]
  fn parse_bool(&mut self) -> Result<bool> {
    // FIXME: to_string below is inefficient, though only on error path
    match self.parse_token()?.as_ref() {
      "Yes" => Ok(true),
      "No" => Ok(false),
      t => Err(Error::UnexpectedToken(t.to_string()))
    }
  }

  #[must_use]
  fn parse_char(&mut self) -> Result<char> {
    match self.parse_token()? {
      ref t if t.len() == 1 => { Ok(t.chars().next().unwrap()) },
      t => Err(Error::UnexpectedToken(t))
    }
  }

  #[must_use]
  fn parse_integer<T: num::Integer>(&mut self) -> Result<T> where T::FromStrRadixErr: std::fmt::Display {
    T::from_str_radix(self.parse_token()?.as_ref(), 10).map_err(|e|Error::ParseNumberError(e.to_string()))
  }

  #[must_use]
  fn parse_newline(&mut self) -> Result<()> {
    if let None = self.peek_char() {
      match self.input.next() {
        Some(Ok(x)) => {
          self.line_current = x;
          self.line_index += 1;
          Ok(())
        },
        Some(Err(e)) => Err(Error::Io(e)),
        None => Err(Error::UnexpectedEof),
      }
    } else {
      Err(Error::TrailingLineCharacters)
    }
  }

  #[must_use]
  fn parse_real<T: num::Float>(&mut self) -> Result<T> where T::FromStrRadixErr: std::fmt::Debug {
    T::from_str_radix(self.parse_token()?.as_ref(), 10).map_err(|e|Error::ParseNumberError(format!("{:?}", e)))
  }

  #[must_use]
  fn parse_separators(&mut self, i_count: usize) -> Result<()> {
    for _ in 0..i_count {
      // FIXME: Make this use the separator value
      match self.line_current.chars().next() {
        Some(x) if x == self.separator => {
          self.line_current.remove(0);
          continue
        },
        Some(c) => {
          return Err(Error::UnexpectedCharacter(c))
        },
        None => return Err(Error::UnexpectedEol),
      }
    }

    Ok(())
  }

  #[must_use]
  fn parse_sequence<'a>(&'a mut self) -> Sequence<'a, R> {
    Sequence::new(self.separator, self.sequence_style, self)
  }

  #[must_use]
  fn parse_token(&mut self) -> Result<String> {
    let mut old_line_current = String::new();
    std::mem::swap(&mut old_line_current, &mut self.line_current);
    match old_line_current.find(self.separator) {
      Some(index) => {
        self.line_current = old_line_current.split_off(index);
        Ok(old_line_current)
      },
      None if old_line_current.is_empty() => Err(Error::UnexpectedEol),
      None => {
        Ok(old_line_current)
      }
    }
  }

  #[must_use]
  fn parse_tuple<'a>(&'a mut self, i_length: usize) -> Tuple<'a, R> {
    Tuple::new(i_length, self)
  }

  fn set_sequence_style(&mut self, i_style: SequenceStyle) {
    self.sequence_style = i_style;
  }
}

struct Tuple<'a, R: std::io::BufRead + 'a> {
  de : &'a mut Deserializer<R>,
  length: usize,
  index: usize,
}

impl<'a, R: std::io::BufRead> Tuple<'a, R> {
  fn new(i_count: usize, i_de: &'a mut Deserializer<R>) -> Self {
    Tuple {
      de: i_de,
      index: 0,
      length: i_count
    }
  }
}

impl<'a, 'de: 'a, R: std::io::BufRead + 'a> serde::de::SeqAccess<'de> for Tuple<'a, R> {
  type Error = Error;

  fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
    if self.index >= self.length {
      Ok(None)
    } else {
      self.index += 1;
      seed.deserialize(&mut *self.de).map(Some)
    }
  }
}

#[derive(Clone,Copy)]
enum SequenceStyle {
  Following,
  FollowingSkipLast,
  Preceding,
}

struct Sequence<'a, R: std::io::BufRead + 'a> {
  de: &'a mut Deserializer<R>,
  first: bool,
  separator: char,
  style: SequenceStyle,
}

impl<'a, R: std::io::BufRead> Sequence<'a, R> {
  fn new(i_separator: char, i_style: SequenceStyle, i_de: &'a mut Deserializer<R>) -> Self {
    Sequence {
      de: i_de,
      first: true,
      separator: i_separator,
      style: i_style,
    }
  }
}

impl<'a, 'de: 'a, R: std::io::BufRead + 'a> serde::de::SeqAccess<'de> for Sequence<'a, R> {
  type Error = Error;

  fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {

    match self.style {
      SequenceStyle::Following => {
        if !self.first { self.de.parse_separators(1)?; };
        if self.de.peek_newline() { return Ok(None) };
      },
      SequenceStyle::FollowingSkipLast => {
        if self.de.peek_newline() { return Ok(None) };
        if !self.first { self.de.parse_separators(1)? };
      },
      SequenceStyle::Preceding => {
        if self.de.peek_newline() { return Ok(None) };
        self.de.parse_separators(1)?;
      }
    }
    self.first = false;
    seed.deserialize(&mut *self.de).map(Some)
  }
}

impl<'de, R: std::io::BufRead> serde::de::MapAccess<'de> for Deserializer<R> {
  type Error = Error;

  fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
    match self.parse_token()?.as_ref() {
      "***End_of_Header***" => Ok(None),
      t => seed.deserialize(t.into_deserializer()).map(Some)
    }
  }

  fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
    self.parse_separators(1)?;
    let r = seed.deserialize(&mut *self);
    self.parse_newline()?;
    r
  }
}

impl<'de, 'a, R: std::io::BufRead> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
  type Error = Error;

  fn deserialize_any<V: serde::de::Visitor<'de>>(self, _: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_bool<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_bool(self.parse_bool()?)
  }

  fn deserialize_byte_buf<V: serde::de::Visitor<'de>>(self, _: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_bytes<V: serde::de::Visitor<'de>>(self, _: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_char<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_char(self.parse_char()?)
  }

  fn deserialize_enum<V: serde::de::Visitor<'de>>(self, _name: &'static str, _variants: &'static [&'static str], v: V) -> Result<V::Value> {
    v.visit_enum(self.parse_token()?.into_deserializer())
  }

  fn deserialize_f32<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_f32(self.parse_real::<f32>()?)
  }

  fn deserialize_f64<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_f64(self.parse_real::<f64>()?)
  }

  fn deserialize_i8<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_i8(self.parse_integer::<i8>()?)
  }

  fn deserialize_i16<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_i16(self.parse_integer::<i16>()?)
  }

  fn deserialize_i32<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_i32(self.parse_integer::<i32>()?)
  }

  fn deserialize_i64<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_i64(self.parse_integer::<i64>()?)
  }

  fn deserialize_ignored_any<V: serde::de::Visitor<'de>>(self, _: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_seq<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_seq(self.parse_sequence())
  }

  fn deserialize_u8<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_u8(self.parse_integer::<u8>()?)
  }

  fn deserialize_u16<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_u16(self.parse_integer::<u16>()?)
  }

  fn deserialize_u32<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_u32(self.parse_integer::<u32>()?)
  }

  fn deserialize_u64<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_u64(self.parse_integer::<u64>()?)
  }

  fn deserialize_option<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    if self.peek_newline() {
      v.visit_none()
    } else {
      v.visit_some(self)
    }
  }

  fn deserialize_map<V: serde::de::Visitor<'de>>(self, _v: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_struct<V: serde::de::Visitor<'de>>(mut self, _name: &'static str, _fields: &'static [&'static str], v: V) -> Result<V::Value> {
    let value = v.visit_map(&mut self);
    self.parse_separators(1)?;
    value
  }

  fn deserialize_identifier<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_string(self.parse_token()?)
  }

  fn deserialize_newtype_struct<V: serde::de::Visitor<'de>>(self, _name: &'static str, v: V) -> Result<V::Value> {
    v.visit_newtype_struct(self)
  }

  fn deserialize_str<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_string(self.parse_token()?)
  }

  fn deserialize_string<V: serde::de::Visitor<'de>>(self, v: V) -> Result<V::Value> {
    v.visit_string(self.parse_token()?)
  }

  fn deserialize_tuple<V: serde::de::Visitor<'de>>(self, len: usize, v: V) -> Result<V::Value> {
    v.visit_seq(self.parse_tuple(len))
  }

  fn deserialize_tuple_struct<V: serde::de::Visitor<'de>>(self, _name: &'static str, _len: usize, _v: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_unit<V: serde::de::Visitor<'de>>(self, _v: V) -> Result<V::Value> {
    unimplemented!()
  }

  fn deserialize_unit_struct<V: serde::de::Visitor<'de>>(self, _name: &'static str, _v: V) -> Result<V::Value> {
    unimplemented!()
  }
}

pub fn from_reader<R: std::io::BufRead>(i_reader: R) -> Result<File> {
  use serde::Deserialize;
  let mut deserializer = Deserializer::from_reader(i_reader)?;


  let lvm_file = File {
    header: FileHeader::deserialize(&mut deserializer)?,
    measurements: {
      deserializer.parse_newline()?;
      deserializer.parse_separators(1)?;

      let mut measurements = vec![];
      loop {
        match deserializer.parse_newline() {
          Ok(()) => {},
          Err(Error::UnexpectedEof) => break,
          Err(e) => return Err(e),
        }
        deserializer.set_sequence_style(SequenceStyle::Following);
        let measurement_header = MeasurementHeader::deserialize(&mut deserializer).unwrap();
        deserializer.parse_separators(measurement_header.channels.0)?;
        deserializer.parse_newline()?;

        deserializer.set_sequence_style(SequenceStyle::FollowingSkipLast);
        let data_headings = Vec::<String>::deserialize(&mut deserializer)?;
        deserializer.parse_newline()?;

        deserializer.set_sequence_style(SequenceStyle::Preceding);
        let mut data_rows = vec![];
        loop {
          if deserializer.peek_newline() { break; }
          let data_row = DataRow::deserialize(&mut deserializer)?;
          deserializer.parse_newline()?;
          data_rows.push(data_row);
        }

        measurements.push(Measurement {
          header: measurement_header,
          data_headings: data_headings,
          data: data_rows,
        });
      }
      measurements
    }
  };

  Ok(lvm_file)
}

#[cfg(test)]
mod tests {
  #[test]
  fn lvm_parsing() {
    super::super::env_logger::init().unwrap();
    let file_reader = super::std::fs::File::open("../data.lvm").unwrap();
    let buf_reader = super::std::io::BufReader::new(file_reader);
    let lvm_file = super::from_reader(buf_reader).unwrap();
    info!("{:#?}", lvm_file.header);
    for measurement in lvm_file.measurements {
      info!("{:#?}", measurement.header);
    }
  }
}
