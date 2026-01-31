//! Native GUI module using egui/eframe.
//!
//! Provides a simple GUI runtime for ASG applications.

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use std::collections::HashMap;

#[cfg(feature = "gui")]
use crate::interpreter::Value;

/// Represents a GUI widget description from ASG
#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub enum Widget {
    Window {
        title: String,
        width: f32,
        height: f32,
        children: Vec<Widget>,
    },
    VBox {
        children: Vec<Widget>,
    },
    HBox {
        children: Vec<Widget>,
    },
    Label {
        text: String,
    },
    Button {
        text: String,
        // Callback будет храниться отдельно
    },
    TextField {
        id: String,
        value: String,
    },
    Canvas {
        width: f32,
        height: f32,
    },
}

/// GUI Application state
#[cfg(feature = "gui")]
pub struct ASGGuiApp {
    pub title: String,
    pub widgets: Vec<Widget>,
    pub text_fields: HashMap<String, String>,
    pub result: Option<String>,
}

#[cfg(feature = "gui")]
impl Default for ASGGuiApp {
    fn default() -> Self {
        Self {
            title: "ASG App".to_string(),
            widgets: Vec::new(),
            text_fields: HashMap::new(),
            result: None,
        }
    }
}

#[cfg(feature = "gui")]
impl ASGGuiApp {
    pub fn new(title: &str, widgets: Vec<Widget>) -> Self {
        Self {
            title: title.to_string(),
            widgets,
            text_fields: HashMap::new(),
            result: None,
        }
    }

    /// Convert ASG Value to Widget tree
    pub fn value_to_widget(val: &Value) -> Option<Widget> {
        match val {
            Value::Dict(d) => {
                let widget_type = match d.get("type") {
                    Some(Value::String(s)) => s.as_str(),
                    _ => return None,
                };

                let children = match d.get("children") {
                    Some(Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| Self::value_to_widget(v))
                        .collect(),
                    _ => Vec::new(),
                };

                match widget_type {
                    "GuiWindow" => {
                        let title = match children.get(0) {
                            Some(Widget::Label { text }) => text.clone(),
                            _ => "Window".to_string(),
                        };
                        Some(Widget::Window {
                            title,
                            width: 400.0,
                            height: 300.0,
                            children: children.into_iter().skip(3).collect(),
                        })
                    }
                    "GuiVBox" => Some(Widget::VBox { children }),
                    "GuiHBox" => Some(Widget::HBox { children }),
                    "GuiLabel" => {
                        let text = match children.get(0) {
                            Some(Widget::Label { text }) => text.clone(),
                            _ => "".to_string(),
                        };
                        Some(Widget::Label { text })
                    }
                    "GuiButton" => {
                        let text = match children.get(0) {
                            Some(Widget::Label { text }) => text.clone(),
                            _ => "Button".to_string(),
                        };
                        Some(Widget::Button { text })
                    }
                    "GuiTextField" => Some(Widget::TextField {
                        id: "input".to_string(),
                        value: String::new(),
                    }),
                    _ => None,
                }
            }
            Value::String(s) => Some(Widget::Label { text: s.clone() }),
            Value::Int(n) => Some(Widget::Label {
                text: n.to_string(),
            }),
            Value::Float(f) => Some(Widget::Label {
                text: f.to_string(),
            }),
            _ => None,
        }
    }

    fn render_widget(&mut self, ui: &mut egui::Ui, widget: &Widget) {
        match widget {
            Widget::Label { text } => {
                ui.label(text);
            }
            Widget::Button { text } => {
                if ui.button(text).clicked() {
                    // Button clicked
                    self.result = Some(format!("Button '{}' clicked", text));
                }
            }
            Widget::TextField { id, value: _ } => {
                let text = self.text_fields.entry(id.clone()).or_insert_with(String::new);
                ui.text_edit_singleline(text);
            }
            Widget::VBox { children } => {
                ui.vertical(|ui| {
                    for child in children {
                        self.render_widget(ui, child);
                    }
                });
            }
            Widget::HBox { children } => {
                ui.horizontal(|ui| {
                    for child in children {
                        self.render_widget(ui, child);
                    }
                });
            }
            Widget::Canvas { width, height } => {
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(*width, *height),
                    egui::Sense::hover(),
                );
                painter.rect_filled(
                    response.rect,
                    0.0,
                    egui::Color32::from_rgb(30, 30, 50),
                );
            }
            Widget::Window { children, .. } => {
                for child in children {
                    self.render_widget(ui, child);
                }
            }
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for ASGGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.title);
            ui.separator();

            let widgets = self.widgets.clone();
            for widget in &widgets {
                self.render_widget(ui, widget);
            }

            if let Some(result) = &self.result {
                ui.separator();
                ui.label(format!("Result: {}", result));
            }
        });
    }
}

/// Run a GUI application
#[cfg(feature = "gui")]
pub fn run_gui(title: &str, widgets: Vec<Widget>) -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title(title),
        ..Default::default()
    };

    eframe::run_native(
        title,
        options,
        Box::new(|_cc| Ok(Box::new(ASGGuiApp::new(title, widgets)))),
    )
    .map_err(|e| e.to_string())
}

/// Calculator GUI specifically
#[cfg(feature = "gui")]
pub fn run_calculator() -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 400.0])
            .with_title("ASG Calculator"),
        ..Default::default()
    };

    eframe::run_native(
        "ASG Calculator",
        options,
        Box::new(|_cc| Ok(Box::new(CalculatorApp::default()))),
    )
    .map_err(|e| e.to_string())
}

#[cfg(feature = "gui")]
#[derive(Default)]
struct CalculatorApp {
    display: String,
    result: f64,
}

#[cfg(feature = "gui")]
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ASG Calculator");
            ui.separator();

            // Display
            ui.add(
                egui::TextEdit::singleline(&mut self.display)
                    .font(egui::TextStyle::Heading)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(10.0);

            // Buttons grid
            egui::Grid::new("calc_buttons")
                .spacing([5.0, 5.0])
                .show(ui, |ui| {
                    let buttons = [
                        ["C", "(", ")", "/"],
                        ["7", "8", "9", "*"],
                        ["4", "5", "6", "-"],
                        ["1", "2", "3", "+"],
                        ["0", ".", "=", ""],
                    ];

                    for row in buttons {
                        for btn in row {
                            if btn.is_empty() {
                                ui.label("");
                            } else {
                                let size = egui::Vec2::new(50.0, 40.0);
                                if ui
                                    .add_sized(size, egui::Button::new(btn))
                                    .clicked()
                                {
                                    match btn {
                                        "C" => {
                                            self.display.clear();
                                            self.result = 0.0;
                                        }
                                        "=" => {
                                            // Simple eval
                                            if let Ok(r) = self.eval_expression(&self.display) {
                                                self.result = r;
                                                self.display = r.to_string();
                                            } else {
                                                self.display = "Error".to_string();
                                            }
                                        }
                                        _ => {
                                            self.display.push_str(btn);
                                        }
                                    }
                                }
                            }
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(10.0);
            ui.label(format!("Result: {}", self.result));
            ui.separator();
            ui.label("Powered by ASG v0.7.0");
        });
    }
}

#[cfg(feature = "gui")]
impl CalculatorApp {
    fn eval_expression(&self, expr: &str) -> Result<f64, String> {
        // Simple expression parser
        let expr = expr.replace(' ', "");
        self.parse_expr(&expr, 0).map(|(v, _)| v)
    }

    fn parse_expr(&self, s: &str, pos: usize) -> Result<(f64, usize), String> {
        let (mut left, mut pos) = self.parse_term(s, pos)?;

        while pos < s.len() {
            let c = s.chars().nth(pos).unwrap_or(' ');
            if c == '+' || c == '-' {
                let (right, new_pos) = self.parse_term(s, pos + 1)?;
                if c == '+' {
                    left += right;
                } else {
                    left -= right;
                }
                pos = new_pos;
            } else {
                break;
            }
        }

        Ok((left, pos))
    }

    fn parse_term(&self, s: &str, pos: usize) -> Result<(f64, usize), String> {
        let (mut left, mut pos) = self.parse_factor(s, pos)?;

        while pos < s.len() {
            let c = s.chars().nth(pos).unwrap_or(' ');
            if c == '*' || c == '/' {
                let (right, new_pos) = self.parse_factor(s, pos + 1)?;
                if c == '*' {
                    left *= right;
                } else {
                    left /= right;
                }
                pos = new_pos;
            } else {
                break;
            }
        }

        Ok((left, pos))
    }

    fn parse_factor(&self, s: &str, pos: usize) -> Result<(f64, usize), String> {
        if pos >= s.len() {
            return Err("Unexpected end".to_string());
        }

        let c = s.chars().nth(pos).unwrap();
        if c == '(' {
            let (val, new_pos) = self.parse_expr(s, pos + 1)?;
            if s.chars().nth(new_pos) != Some(')') {
                return Err("Missing )".to_string());
            }
            return Ok((val, new_pos + 1));
        }

        // Parse number
        let mut end = pos;
        while end < s.len() {
            let ch = s.chars().nth(end).unwrap();
            if ch.is_ascii_digit() || ch == '.' {
                end += 1;
            } else {
                break;
            }
        }

        let num_str = &s[pos..end];
        let num: f64 = num_str.parse().map_err(|_| "Invalid number")?;
        Ok((num, end))
    }
}
