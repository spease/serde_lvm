use super::errors::*;
use chrono;
use itertools::Itertools;
use semver;
use serde;
use std;

/// The default is the decimal separator of the system.
/// 
/// Symbol used to separate the integral part of a number from the fractional part.
/// A decimal separator usually is a dot or a comma.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum DecimalSeparator {
  #[serde(rename=".")]
  /// Dot character, ASCII \0x2E
  Dot,

  /// Comma character, ASCII \0x2C
  #[serde(rename=",")]
  Comma,
}

macro_rules! wrapper_classes {
    ($($(#[$attr:meta])* pub struct $s:ident($t:ty);)*) => {
        $(
            $(#[$attr])*
            #[derive(Clone, Debug, Deserialize, Display, Eq, From, Into, PartialEq, PartialOrd, Serialize, Shrinkwrap)]
            pub struct $s($t);
        )*
    }
}

wrapper_classes!(
    /// Channel name
    pub struct ChannelName(String);
    /// Name or instrument class of the unit under test
    pub struct InstrumentName(String);
    /// Model number of a unit under test
    pub struct ModelNumber(String);
    /// Serial number of the unit under test
    pub struct SerialNumber(String);
    /// Name of the operator who collected the data
    pub struct OperatorName(String);
    /// Name of the project that data was for
    pub struct ProjectName(String);
    /// Name of a test
    pub struct TestName(String);
    /// Test number from a TestSeries
    pub struct TestNumber(String);
    /// Series of a test
    pub struct TestSeries(String);
);

/// Test numbers in a TestSeries
#[derive(Clone, Debug, Shrinkwrap)]
pub struct TestNumbers(Vec<TestNumber>);

// FIXME: Add support for comma separator too
const TEST_NUMBERS_SEPARATOR: char = ';';

impl std::str::FromStr for TestNumbers {
  type Err = ();

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(TestNumbers(s.split(TEST_NUMBERS_SEPARATOR).map(|x|TestNumber(x.to_owned())).collect()))
  }
}
impl<'de> serde::de::Deserialize<'de> for TestNumbers {
  fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
    deserializer.deserialize_str(TestNumbersVisitor)
  }
}
impl std::fmt::Display for TestNumbers {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    // FIXME: Could be more efficient
    f.write_str(&self.0.iter().map(|x|&x.0).join(&TEST_NUMBERS_SEPARATOR.to_string()))
  }
}

impl serde::ser::Serialize for TestNumbers {
  fn serialize<S: serde::ser::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.collect_str(self)
  }
}

struct TestNumbersVisitor;

impl<'de> serde::de::Visitor<'de> for TestNumbersVisitor {
  type Value = TestNumbers;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("Test numbers separated by semicolons")
  }

  fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
    value.parse().map_err(|_|serde::de::Error::custom(""))
  }
}

pub(super) type DataRow = (Vec<f64>, Option<String>);

/// Timezone-dependent date
#[derive(Clone, Copy, Debug, Eq, From, Into, Ord, PartialEq, PartialOrd, Shrinkwrap)]
#[must_use]
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
#[must_use]
pub struct File {
  /// Metadata on the file itself
  pub header: FileHeader,
  /// Measurement segments
  pub measurements: Vec<Measurement>,
}

/// Header for the file
#[derive(Debug, Deserialize, Serialize)]
#[must_use]
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
  decimal_separator: DecimalSeparator,

  /// Specifies whether each packet has a header.
  #[serde(default, rename="Multi_Headings")]
  multi_headings: bool,

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
  separator: Separator,

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

/// A set of measurements
#[derive(Debug, Deserialize, Serialize)]
#[must_use]
pub struct Measurement {
  /// Header for this measurement segment
  pub header: MeasurementHeader,
  /// Headings for data columns
  pub data_headings: Vec<String>,
  /// Data columns
  pub data: Vec<DataRow>,
}

/// Header for measurement data
#[derive(Debug, Deserialize, Serialize)]
#[must_use]
pub struct MeasurementHeader {
  /// Number of channels in the packet.
  ///
  /// This field must occur before any fields that depend on it.
  /// For example, the Samples field has entries for each channel,
  /// so the reader must know the number of channels to properly parse it.
  #[serde(rename="Channels")]
  pub channels: (usize, Vec<ChannelName>),

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
  pub test_name: Option<TestName>,

  /// Test numbers in the Test_Series that acquired the data in this segment.
  #[serde(rename="Test_Number")]
  pub test_numbers: Option<TestNumbers>,

  /// Series of the test performed to get the data in this packet.
  #[serde(rename="Test_Series")]
  pub test_series: Option<TestSeries>,

  /// Time of day when you started acquiring the data set in the segment.
  ///
  /// Each data set includes a different time.
  /// The times are placed in the same column as the y data of the data set.
  #[serde(rename="Time")]
  pub time: Vec<Time>,

  /// Model number of the unit under test.
  #[serde(rename="UUT_M/N")]
  pub uut_mn: Option<ModelNumber>,

  /// Name or instrument class of the unit under test.
  #[serde(rename="UUT_Name")]
  pub uut_name: Option<InstrumentName>,

  /// Serial number of the unit under test.
  #[serde(rename="UUT_S/N")]
  pub uut_sn: Option<SerialNumber>,

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

  /// Labels for the units used in plotting the y data.
  ///
  /// The label appears in the same column as the y data to which it corresponds.
  /// You do not have to fill in all unit labels.
  #[serde(rename="Y_Unit_Label")]
  pub y_unit_label: Option<Vec<Unit>>,
}

/// Character(s) used to separate each field in the file
#[derive(AsRefStr, Clone, Copy, Debug, Deserialize, Serialize)]
#[must_use]
pub enum Separator {
  /// Comma separator (ASCII \0x2C)
  Comma,
  /// Tab separator (ASCII \0x09)
  Tab,
}

impl Separator {
  pub(crate) fn try_from(i_char: char) -> Result<Separator> {
    match i_char {
      ',' => Ok(Separator::Comma),
      '\t' => Ok(Separator::Tab),
      c => Err(ErrorKind::InvalidSeparator(c).into())
    }
  }
}

impl From<Separator> for char {
  fn from(s: Separator) -> char {
    match s {
      Separator::Comma => ',',
      Separator::Tab => '\t',
    }
  }
}

impl Default for Separator {
  fn default() -> Self { Separator::Tab }
}

/// Timezone-dependent time
#[derive(Clone, Copy, Debug, Eq, From, Into, Ord, PartialEq, PartialOrd, Shrinkwrap)]
#[must_use]
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
    s.collect_str(self)
  }
}

#[must_use]
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


/// Format of axis values - absolute or relative
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[must_use]
pub enum TimePref {
  /// x-value is number of seconds since midnight, January 1, 1904 GMT
  Absolute,

  /// x-value is number of seconds since the date and time stamps
  Relative,
}

impl Default for TimePref {
  fn default() -> Self { TimePref::Relative }
}

/// Label for an axis
//FIXME: Should probable be an "arbitrary text string"
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[must_use]
pub enum Unit {
  /// Milliamps
  Milliamps,
  /// Volts
  Volts,
}

/// Specifies the unit type of an axis
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[must_use]
pub enum UnitType {
  /// Electric Potential (Jouls)
  #[serde(rename="Electric_Potential")]
  ElectricPotential,

  /// Time (seconds)
  Time,
}

impl Default for UnitType {
  fn default() -> Self { UnitType::ElectricPotential }
}

/// Reader / writer version
#[derive(Clone, Debug, Eq, From, Into, Ord, PartialEq, PartialOrd, Shrinkwrap)]
#[must_use]
pub struct Version(semver::Version);

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    if self.0.patch != 0 || !self.0.pre.is_empty() || !self.0.build.is_empty() {
      Err(std::fmt::Error)
    } else {
       write!(f, "{}.{}", self.0.major, self.0.minor)
    }
  }
}

impl std::str::FromStr for Version {
  type Err = semver::SemVerError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    if !s.contains('.') {
      semver::Version::parse(&format!("{}.0.0", s)).map(Version)
    } else {
      semver::Version::parse(s).map(Version)
    }
  }
}

impl<'de> serde::de::Deserialize<'de> for Version {
  fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
    deserializer.deserialize_str(VersionVisitor)
  }
}

impl serde::ser::Serialize for Version {
  fn serialize<S: serde::ser::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(self.to_string().as_ref())
  }
}

#[must_use]
struct VersionVisitor;

impl<'de> serde::de::Visitor<'de> for VersionVisitor {
  type Value = Version;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("A version with the format major.minor")
  }

  fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
    use std::str::FromStr;
    Self::Value::from_str(value).map_err(serde::de::Error::custom)
  }
}

///  Specifies which x-values are saved.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[must_use]
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

