use reference::Reference;
use rustorm::Column;
use rustorm::Table;
use rustorm::types::SqlType;

#[derive(Debug, Serialize, Clone)]
pub enum Widget {
    Textbox,
    UuidTextbox,
    Password,
    TagSelection,
    MultilineText,
    MarkdownHtml,
    CodeHighlighter,
    ColorSelector,
    DatePicker,
    DateTimePicker,

    LogoImage,
    MediumImage,
    LargeImageEmbed,

    /// dropdown where there is no need
    /// to fetch for more data
    /// for enums
    /// where there is only
    /// a few choices
    FixDropdown(Vec<String>),
    Radiogroup(Vec<String>),
    Checkboxgroup(Vec<String>),

    Dropdown,
    DropdownWithImage,
    AutocompleteDropdown,
    DialogDropdown,
    TableLookupDropdown,

    Checkbox,
    CheckmarkStatusImage, // use check mark such as for "is_active"
    IndicatorStatusImage, // on/off - dull gray/ birght green LED
    ToggleButton,         // switch button with on/off
    PrimaryUrlLink,
    UrlLink,
    UrlTextbox,

    VideoLink,
    YoutubeVideoEmbed,
    TweetEmbed,

    PrimaryButton,
    SecondaryButton,
    AuxilliaryButton,

    FileDownloadLink,
    FileUpload,
    Maplookup,
    CountryList,
    CountryListWithFlag,
    TimezoneLookup,

    PdfViewer,
    ExcelViewer,
    CsvRenderer,
    VideoPlayer,
    AudioPlayer,

    Viewer3D,
}


/// contains the widget
/// and the dropdown data
#[derive(Debug, Serialize, Clone)]
pub struct ControlWidget {
    widget: Widget,

    /// if the widget is Dropdown, DropdownWithImage, AutoCompleteDropdown
    /// DialogDropdown, CountryList, CountrListWithFlag
    dropdown_data: Option<DropdownData>,

    /// width (character wise) of the widget based on
    /// average of the database values on this column
    width: i32,

    /// if limit is set in column this will warn the user
    /// if the value is too long
    max_len: Option<i32>,

    /// height of the control, character wise
    /// textbox defaults to 1
    height: i32,

    /// text-align left align for text, right align for decimal values
    /// boolean values align center
    alignment: Alignment,
}


#[derive(Debug, Serialize, Clone)]
pub enum Alignment {
    Left,
    Right,
    Center,
}


/// a simple downdown list in string
#[derive(Debug, Serialize, Clone)]
pub struct DropdownRecord {
    identifier: String,
    display: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DropdownList {
    /// api url for the next page to be loaded
    api_url: String,
    /// the selected value of the record
    selected: Option<DropdownRecord>,
    /// the selection, autoloads on scroll till reaches the last page
    selection: Vec<DropdownRecord>,
    /// whether or not all the items of the page has been loaded
    reached_last_page: bool,
}

#[derive(Debug, Serialize, Clone)]
pub enum Image {
    Url(String),
    DataUrl(String),
    /// image type, blob
    Blob(String, Vec<u8>),
    CssClass(String),
}


#[derive(Debug, Serialize, Clone)]
pub struct DropdownRecordWithImage {
    identifier: String,
    display: String,
    /// the url image of the record display
    image: Image,
}

#[derive(Debug, Serialize, Clone)]
pub struct DropdownListWithImage {
    /// api url for the next page to be loaded
    api_url: String,
    /// the selected value of the record
    selected: Option<DropdownRecordWithImage>,
    /// the selection, autoloads on scroll till reaches the last page
    choices: Vec<DropdownRecordWithImage>,
    /// whether or not all the items of the page has been loaded
    reached_last_page: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DropdownListWithAutocomplete {
    /// api url for the next page to be loaded
    api_url: String,
    /// the selected value of the record
    selected: Option<DropdownRecord>,
    /// the selection, autoloads on scroll till reaches the last page
    choices: Vec<DropdownRecord>,
    /// whether or not all the items of the page has been loaded
    reached_last_page: bool,
}


#[derive(Debug, Serialize, Clone)]
pub enum DropdownData {
    DropdownList(DropdownList),
    /// whatever the image shape displayed as is
    DropdownListWithImage(DropdownListWithImage),
    /// images in rounded corner
    DropdownListWithRoundedImage(DropdownListWithImage),
    DropdownListWithAutocomplete(DropdownListWithAutocomplete),
}


impl ControlWidget {
    /// derive widget base from column
    /// reference is derived first then the widget is based
    /// from the reference
    pub fn derive_control_widget(column: &Column, reference: &Option<Reference>) -> ControlWidget {
        let limit = column.specification.get_limit();
        let alignment = Self::derive_alignment(column);
        let sql_type = &column.specification.sql_type;
        let width = Self::get_width(column).unwrap_or(20);
        if let Some(ref reference) = *reference {
            let widget = reference.get_widget_fullview();
            ControlWidget {
                widget,
                dropdown_data: None,
                width,
                max_len: limit,
                height: 1,
                alignment,
            }
        } else {
            let widget = if *sql_type == SqlType::Bool {
                Widget::Checkbox
            } else if *sql_type == SqlType::TimestampTz || *sql_type == SqlType::Timestamp {
                Widget::DateTimePicker
            } else if *sql_type == SqlType::Date {
                Widget::DatePicker
            } else if *sql_type == SqlType::Uuid {
                Widget::UuidTextbox
            } else {
                Widget::Textbox
            };
            ControlWidget {
                widget,
                dropdown_data: None,
                width,
                max_len: limit,
                height: 1,
                alignment,
            }
        }
    }

    fn get_width(column: &Column) -> Option<i32> {
        let sql_type = &column.specification.sql_type;
        if let Some(ref stat) = column.stat {
            Some(stat.avg_width)
        } else if *sql_type == SqlType::Uuid {
            Some(36)
        } else {
            None
        }
    }

    pub fn from_has_one_table(columns: &Vec<&Column>, _table: &Table) -> Self {
        let reference = Reference::TableLookup;
        let widget = reference.get_widget_fullview();
        let width = columns
            .iter()
            .map(|col| match Self::get_width(col) {
                Some(width) => width,
                None => 0,
            })
            .max()
            .unwrap_or(0);

        ControlWidget {
            widget,
            dropdown_data: None, // not yet computed here
            width,
            max_len: None,
            height: 1,
            alignment: Alignment::Left,
        }
    }

    fn derive_alignment(column: &Column) -> Alignment {
        let sql_type = &column.specification.sql_type;
        match *sql_type {
            SqlType::Bool => Alignment::Center,
            SqlType::Tinyint
            | SqlType::Smallint
            | SqlType::Int
            | SqlType::Bigint
            | SqlType::Real
            | SqlType::Float
            | SqlType::Double
            | SqlType::Numeric => Alignment::Right,

            SqlType::Uuid
            | SqlType::Date
            | SqlType::Timestamp
            | SqlType::TimestampTz
            | SqlType::Time
            | SqlType::TimeTz => Alignment::Right,
            _ => Alignment::Left,
        }
    }
}
