use anyhow::{Context, Result};
use backend::{calculate_capital_gains, CalculatorType, Currency, ReaderType, TransactionsFile};
use druid::commands::OPEN_FILE;
use druid::im::{HashMap as ImHashMap, Vector};
use druid::widget::{
    Align, Button, Container, Flex, Label, LineBreaking, List, Padding,
    RadioGroup, SizedBox, ViewSwitcher, WidgetExt,
};
use druid::{
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, EventCtx, FileDialogOptions,
    FileSpec, FontDescriptor, FontFamily, FontWeight, Handled, Insets, Lens, LocalizedString,
    Target, UnitPoint, Widget, WindowDesc,
};
use druid_widget_nursery::dropdown::DROPDOWN_SHOW;
use druid_widget_nursery::Dropdown;
use itertools::izip;
use std::path::PathBuf;
use std::str::FromStr;

const HORIZONTAL_WIDGET_SPACING: f64 = 5.0;
const VERTICAL_WIDGET_SPACING: f64 = 20.0;

const TITLE: &str = "Australian Crypto Capital Gains Calculator";
const WINDOW_TITLE: LocalizedString<InitialState> = LocalizedString::new(TITLE);

#[derive(Clone, Data, Lens)]
struct InitialState {
    data_sources: Vector<DataPickerState>,
    calculator_type: String,
    // Only one of these two should be Some at the same time.
    // I would represent this differently, like as this:
    // Option<Result<HashMap<Currency, f64>>>
    // But the Data trait mandates that everything be cloneable and
    // anyhow::Error is not (and I couldn't get it to work properly
    // at runtime with an Arc).
    capital_gains: Option<ImHashMap<Currency, f64>>,
    error_text: Option<String>,
}

impl InitialState {
    fn new() -> InitialState {
        InitialState {
            data_sources: Vector::new(),
            calculator_type: CalculatorType::Fifo.to_string(),
            capital_gains: None,
            error_text: None,
        }
    }
}

struct FileOpenerDelegate;

impl AppDelegate<InitialState> for FileOpenerDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut InitialState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(OPEN_FILE) {
            let data_picker_state = DataPickerState::new(file_info.path().to_path_buf());
            data.data_sources.push_back(data_picker_state);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn main() -> Result<()> {
    // Describe the main window.
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .window_size((800.0, 600.0));

    // Create the initial app state.
    let initial_state = InitialState::new();

    // Start the application.
    AppLauncher::with_window(main_window)
        .delegate(FileOpenerDelegate)
        .log_to_console()
        .launch(initial_state)
        .context("Failed to launch application")?;

    Ok(())
}

fn build_root_widget() -> impl Widget<InitialState> {
    // Layout that we build throughout this function.
    let mut layout = Flex::column().with_spacer(50.0);

    // Title font and label.
    let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(24.0);

    let label = Label::new(TITLE.to_string()).with_font(font);
    layout = layout
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING);

    // Calculator strategy dropdown.
    let calculator_dropdown = Dropdown::new(
        Button::new(|calculator_type_string: &String, _: &Env| {
            calculator_type_string.to_string().to_uppercase()
        })
        .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
        |_, _| {
            let choices: Vec<(String, String)> = izip!(
                CalculatorType::variants()
                    .iter()
                    .map(|s| s.to_uppercase())
                    .collect::<Vec<String>>(),
                CalculatorType::variants()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            )
            .collect();
            RadioGroup::new(choices)
        },
    )
    .align_left()
    .lens(InitialState::calculator_type);

    layout = layout
        .with_child(
            Flex::row()
                .with_child(Label::new("Capital gains calculation strategy:"))
                .with_spacer(HORIZONTAL_WIDGET_SPACING)
                .with_child(calculator_dropdown),
        )
        .with_spacer(VERTICAL_WIDGET_SPACING);

    // Button for selecting a data source.
    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("CSV file", &["csv"])])
        .name_label("Select")
        .title("Select data source")
        .button_text("Open");

    let open_button = Button::new("Add data source").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    // Button for clearing data sources.
    // The button to calculate the capital gains.
    let clear_sources_button = Button::new("Clear sources")
        .on_click(move |_, data: &mut InitialState, _| {
            data.data_sources = Vector::new();
            data.capital_gains = None;
            data.error_text = None;
        });

    // Add both buttons side by side.
    layout = layout
        .with_child(Flex::row().with_child(open_button).with_spacer(HORIZONTAL_WIDGET_SPACING * 2.0).with_child(clear_sources_button))
        .with_spacer(VERTICAL_WIDGET_SPACING);

    // List of the files we've chosen to use as data sources.
    layout.add_flex_child(
        List::new(|| build_data_picker_widget()).lens(InitialState::data_sources),
        1.0,
    );

    // The button to calculate the capital gains.
    let calculate_button = Button::new("Calculate capital gains")
        .on_click(move |_, data: &mut InitialState, _| {
            // Just calculate in line, it's quick.
            let (capital_gains, error_text) = calculate_capital_gains_local(&data);
            data.capital_gains = capital_gains;
            data.error_text = error_text;
        })
        .disabled_if(|data, _| data.data_sources.is_empty());
    layout = layout
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(calculate_button);

    // If we have results, present those. I think we have to use a List for this.
    // Fortunately the results are in a list format anyway so this works.
    let results_widget = SizedBox::new(ViewSwitcher::new(
        |data: &InitialState, _env| data.clone(),
        move |_, data: &InitialState, _env| {
            if data.capital_gains.is_some() {
                let mut column = Flex::column();
                for (currency, capital_gain) in data.capital_gains.as_ref().unwrap() {
                    column.add_child(Label::new(format!(
                        "Capital gain for {}: ${:.2} AUD",
                        currency.0, capital_gain
                    )));
                }
                Box::new(column)
            } else {
                let mut label = if data.error_text.is_some() {
                    Label::new(format!(
                        "Something went wrong: {}",
                        data.error_text.as_ref().unwrap()
                    ))
                } else {
                    // No data yet, return empty label.
                    Label::new("")
                };
                label = label.with_line_break_mode(LineBreaking::WordWrap);
                Box::new(label)
            }
        },
    )).width(600.0);
    layout = layout.with_spacer(VERTICAL_WIDGET_SPACING).with_child(results_widget);

    // Finally present the widget starting from the top.
    Align::vertical(UnitPoint::TOP, layout)
}

fn calculate_capital_gains_local(
    data: &InitialState,
) -> (Option<ImHashMap<Currency, f64>>, Option<String>) {
    let mut transactions_files = Vec::new();
    for data_picker_state in data.data_sources.iter() {
        let reader_type = ReaderType::from_str(&data_picker_state.reader_type).unwrap();
        let transactions_file = TransactionsFile {
            path: data_picker_state.path.clone(),
            reader_type,
        };
        transactions_files.push(transactions_file);
    }
    let calculator_type = CalculatorType::from_str(&data.calculator_type).unwrap();
    let capital_gains = calculate_capital_gains(transactions_files, calculator_type);
    let (capital_gains, error_text) = match capital_gains {
        Ok(cg) => (Some(ImHashMap::from(&cg)), None),
        Err(e) => (None, Some(format!("{:#}", e))),
    };
    (capital_gains, error_text)
}

/// This struct contains information necessary to load up a single file.
/// If there are multiple files to load, instantiate multiple of these.
#[derive(Clone, Data, Debug, Lens)]
struct DataPickerState {
    /// Path to the file we want to load up.
    #[data(same_fn = "PartialEq::eq")]
    path: PathBuf,
    /// Reader we want to use to load up the file.
    /// Unfortunately we have to use a String here unless we want
    /// to pollute our dependent crates with the Druid dependency too.
    /// This is only necessary because of the dropdown, otherwise
    /// using the same PartialEq hack as for path would work fine.
    reader_type: String,
}

impl DataPickerState {
    fn new(path: PathBuf) -> DataPickerState {
        DataPickerState {
            path,
            reader_type: ReaderType::Coinjar.to_string(),
        }
    }

    fn get_path_end(&self) -> String {
        self.path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap()
    }
}

/// This function builds a widget based on a single DataPickerState.
/// It formats it all in a nice little box.
fn build_data_picker_widget() -> impl Widget<DataPickerState> {
    // Name of the file.
    let path_end_label = Label::new(|data: &DataPickerState, _env: &Env| data.get_path_end());

    // Dropdown for choosing which kind of reader to use.
    let dropdown = Dropdown::new(
        Button::new(|reader_type_string: &String, _: &Env| reader_type_string.to_string())
            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROPDOWN_SHOW)),
        |_, _| {
            let choices: Vec<(&str, String)> = izip!(
                ReaderType::variants(),
                ReaderType::variants()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            )
            .collect();
            RadioGroup::new(choices)
        },
    )
    .align_left()
    .lens(DataPickerState::reader_type);

    Container::new(
        SizedBox::new(Padding::new(
            Insets::uniform_xy(10.0, 10.0),
            Flex::column()
                .with_child(
                    Flex::row()
                        .with_child(Label::new("Data source:"))
                        .with_spacer(HORIZONTAL_WIDGET_SPACING)
                        .with_child(path_end_label),
                )
                .with_child(
                    Flex::row()
                        .with_child(Label::new("Reader to use:"))
                        .with_spacer(HORIZONTAL_WIDGET_SPACING)
                        .with_child(dropdown),
                ),
        ))
        .width(400.0),
    )
    .border(Color::BLACK, 2.0)
}
