#![allow(unused)]

use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

use eframe::egui;

trait Rule {
    fn desc(&self) -> String;
    fn check(&self, data: &Structure<CourseInstance>) -> bool;
}

struct Rules {
    rules: Vec<Box<dyn Rule>>,
}

impl Rules {

}

const INFO_SYMBOL: &str = "ℹ";

#[derive(Debug, Clone)]
struct Structure<T> {
    fields: Vec<Field<T>>,
    num_semesters: usize,
    school: String,
    version: String,
}

impl Structure<Course> {
    fn new(fields: Vec<Field<Course>>, num_semesters: usize, school: impl Into<String>, version: impl Into<String>) -> Self {
        let school = school.into();
        let version = version.into();
        Self { fields, num_semesters, school, version }
    }

    fn as_instance(self) -> Structure<CourseInstance> {
        let fields = self.fields.into_iter().map(|v| v.as_instance(self.num_semesters)).collect();
        Structure {
            fields,
            school: self.school,
            num_semesters: self.num_semesters,
            version: self.version,
        }
    }

    fn courses(&self) -> impl Iterator<Item = &Course> { self.fields.iter().flat_map(|v| v.courses.iter()) }
    fn courses_mut(&mut self) -> impl Iterator<Item = &mut Course> { self.fields.iter_mut().flat_map(|v| v.courses.iter_mut()) }
}

#[derive(Debug, Clone)]
struct Field<T> {
    name: String,
    courses: Vec<T>,
    max_usable: Option<u32>,
}

impl Field<Course> {
    fn new(name: impl Into<String>, courses: Vec<Course>) -> Self {
        let name = name.into();
        let max_usable = None;
        Self { name, courses, max_usable }
    }

    fn as_instance(self, num_semesters: usize) -> Field<CourseInstance> {
        let courses = self.courses.into_iter().map(|v| v.as_instance(num_semesters)).collect();
        Field {
            name: self.name,
            max_usable: self.max_usable,
            courses,
        }
    }

    fn new_max_usable(name: impl Into<String>, subjects: Vec<Course>, max_usable: u32) -> Self {
        let name = name.into();
        let max_usable = Some(max_usable);
        Self { name, courses: subjects, max_usable }
    }
}

impl Field<CourseInstance> {
    fn get_num_semesters(&self) -> u32 { self.courses.iter().map(|v| v.get_num_semesters()).sum() }
    fn get_num_usable_semesters(&self) -> u32 {
        let num_sem = self.get_num_semesters();
        self.max_usable.map(|v| v.min(num_sem)).unwrap_or(num_sem)
    }
}

#[derive(Debug, Clone)]
struct Course {
    name: String,
    id: String,
    tags: HashSet<String>,
    num_semesters: Option<usize>,
    semester_offset: usize,
    // add lessons per week
}

#[derive(Debug)]
struct CourseInstance {
    course: Course,
    semesters: Vec<bool>,
    exam: Option<Exam>,
}

impl CourseInstance {
    fn get_num_semesters(&self) -> u32 { self.semesters.iter().map(|v| *v as u32).sum() }
}

impl Course {
    fn new<S: Into<String>>(name: impl Into<String>, id: impl Into<String>, tags: Vec<S>) -> Self {
        let name = name.into();
        let id = id.into();
        let tags = tags.into_iter().map(|v| v.into()).collect();
        let num_semesters = None;
        let semester_offset = 0;
        Self { name, tags, id, num_semesters, semester_offset }
    }
    
    fn with_semesters(mut self, num_semesters: usize) -> Self {
        self.num_semesters = Some(num_semesters);
        self
    }

    fn with_offset(mut self, offset: usize) -> Self {
        self.semester_offset = offset;
        self
    }

    fn as_instance(self, num_semesters: usize) -> CourseInstance {
        let semesters = (0..num_semesters).map(|_| false).collect();
        CourseInstance {
            course: self,
            semesters,
            exam: None,
        }
    }
}

fn courses() -> Structure<Course> {
    let structure = Structure::new(vec![
        Field::new("1. AF", vec![
            Course::new("Deutsch", "de", vec!["lang", "lk"]),
            Course::new("Englisch", "en", vec!["lang", "lk"]),
            Course::new("Französisch", "frz", vec!["lang", "lk"]),
            Course::new("Latein", "lat", vec!["lang"]),
            Course::new("Russisch", "rus", vec!["lang"]),
            Course::new("Spanisch", "sp", vec!["lang"]),
            Course::new("Musik", "mus", vec!["lk"]),
            Course::new("Kunst", "kun", vec!["lk"]),
            Course::new::<&str>("Darstellendes Spiel", "ds", vec![]),
        ]),
        Field::new("2. AF", vec![
            Course::new("Politikwissenschaft", "pw", vec!["lk"]),
            Course::new("Geschichte", "ges", vec!["lk"]),
            Course::new("Geographie", "geo", vec!["lk"]),
            Course::new::<&str>("Philosophie", "philo", vec![]),
        ]),
        Field::new("3. AF", vec![
            Course::new("Mathematik", "mat", vec!["lk"]),
            Course::new("Physik", "phy", vec!["lk", "nawi"]),
            Course::new("Chemie", "ch", vec!["lk", "nawi"]),
            Course::new("Biologie", "bio", vec!["lk", "nawi"]),
            Course::new::<&str>("Informatik", "inf", vec![]),
        ]),

        Field::new("Sport", vec![
            Course::new::<&str>("Sport", "spo", vec![]),
            Course::new::<&str>("Sport-Theorie", "spo-th", vec![]).with_offset(2),
        ]),

        Field::new_max_usable("Ensemblemusik", vec![
            Course::new::<&str>("Chor", "chor", vec![]),
            Course::new::<&str>("Bläser", "blaeser", vec![]),
        ], 2),
        Field::new_max_usable("Zusatzkurse", vec![
            Course::new::<&str>("CCC", "ccc", vec![]),
            Course::new::<&str>("Debating", "debating", vec![]),
            Course::new::<&str>("Digitale Welten", "dw", vec![]).with_semesters(2),
        ], 2),
        Field::new_max_usable("Seminarkurse", vec![
            Course::new::<&str>("Neurowissenschaften", "neuro", vec![]).with_semesters(2),
            Course::new::<&str>("Doping", "doping", vec![]).with_semesters(2),
            Course::new::<&str>("Finanzmathematik", "fin-mat", vec![]).with_semesters(2),
        ], 2),
        Field::new("Sport", vec![
            Course::new::<&str>("Ski und Snowboard", "ski", vec![]).with_semesters(1).with_offset(1),
        ]),
    ], 4, "Hans-Carossa-Gymnasium", "1.0");
    structure
}

use boa_engine::builtins::map::ordered_map::OrderedMap;
use boa_engine::object::{JsMap, ObjectData};
use boa_engine::prelude::*;
use boa_engine::{Context, object::FunctionBuilder, property::Attribute, builtins::JsArgs};
use boa_gc::{Gc, Cell, Trace};

struct JsRules {
    ctx: Context,
    rules: Gc<Cell<Vec<(String, JsObject, bool)>>>,
}

impl JsRules {
    fn from_js() -> JsResult<Self> {
        let mut ctx = Context::default();

        let js_code = r#"
            rule("Deutsch 4 Semester", () => {
                return "test";
            });

            rule("Anzahl einzubringender Pflichtkurse <= 32", () => {
                return "test";
            }, { optional: true });
        "#;

        let rules = Gc::new(Cell::new(Vec::new()));

        let rule_fn = FunctionBuilder::closure_with_captures(&mut ctx, |_this, args, captures, ctx| {
            let name = args.get(0).ok_or_else(|| ctx.construct_type_error("expected at least 2 arguments but got 0"))?.to_string(ctx)?;
            let func = args.get(1).ok_or_else(|| ctx.construct_type_error("expected at least 2 arguments but got 1"))?.to_object(ctx)?;
            if !func.is_callable() { return Err(ctx.construct_type_error("expected a callable (function)")) }
            let optional = args.get(2).map(|v| if v.is_object() {
                let opt = v.as_object().unwrap();
                let optional = opt.get("optional", ctx).ok().map(|v| if v.is_boolean() { v.as_boolean().unwrap() } else { false }).unwrap_or(false);
                optional
            } else { false }).unwrap_or(false);

            captures.borrow_mut().push((name.to_string(), func, optional));
            Ok(JsValue::undefined())
        }, rules.clone()).name("rule").build();
        ctx.register_global_property("rule", rule_fn, Attribute::all());

        let res = ctx.eval(js_code)?;
        println!("got: {}", res.to_string(&mut ctx).unwrap());
        println!("rules: {:?}", rules.borrow().iter().map(|v| (&v.0, v.2)).collect::<Vec<_>>());

        Ok(Self { ctx, rules })
    }
}



#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Exam {
    Lf1, Lf2,
    Prf3, Prf4,
    Pk5,
}

impl Exam {
    fn list<'a>() -> &'a [Self] {
        &[
            Self::Lf1,
            Self::Lf2,
            Self::Prf3,
            Self::Prf4,
            Self::Pk5,
        ]
    }

    fn filtered<'a>(items: &'a [Self], tags: &'a [String]) -> impl Iterator<Item = &'a Self> {
        std::iter::once(todo!())
    }
}

impl std::fmt::Display for Exam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exam::Lf1 => write!(f, "1. LF"),
            Exam::Lf2 => write!(f, "2. LF"),
            Exam::Prf3 => write!(f, "3. PrF"),
            Exam::Prf4 => write!(f, "4. PrF"),
            Exam::Pk5 => write!(f, "5. PK"),
        }
    }
}

struct App {
    structure: Structure<Course>,
    instance: Option<Structure<CourseInstance>>,
    rules: JsRules,
}

impl App {
    fn new() -> Self {
        let structure = courses();
        let instance = Some(structure.clone().as_instance());
        let rules = JsRules::from_js().expect("could not load rules");
        Self {
            structure,
            instance,
            rules,
        }
    }
}

impl App {
    fn draw_field(ui: &mut egui::Ui, field: &mut Field<CourseInstance>, idx: usize) {
        ui.push_id(idx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading(format!("{}", field.name));
                if let Some(num_sem) = field.max_usable {
                    ui.add(egui::Label::new(egui::RichText::new(INFO_SYMBOL).heading())
                        .sense(egui::Sense::click()))
                        .on_hover_text_at_pointer(format!("max. {} Semester einbringbar", num_sem));
                }
            });

            ui.group(|ui| {
                egui::Grid::new("task_grid").num_columns(3).show(ui, |ui| {
                    for (i, course) in field.courses.iter_mut().enumerate() {
                        Self::draw_course_instance(ui, course, i);
                        ui.end_row();
                    }
                });
            });
        });
    }

    fn draw_grid_info(ui: &mut egui::Ui, num_courses: usize) {
        ui.group(|ui| {
            egui::Grid::new("info_grid").num_columns(3).show(ui, |ui| {
                ui.add_sized((200.0, 20.0), egui::Label::new(format!("Kurs")));
                ui.add_sized((100.0, 20.0), egui::Label::new("PrF"));
                ui.columns(num_courses, |col| for (i, ui) in col.iter_mut().enumerate() {
                    ui.add_sized((ui.available_width(), 20.0), egui::Label::new(format!("{}. Semester", i + 1)));
                });
                ui.end_row();
            });

        });
    }

    fn draw_course_instance(ui: &mut egui::Ui, course: &mut CourseInstance, idx: usize) {
        if ui.add_sized((200.0, 20.0), egui::Label::new(format!("{}", course.course.name)).sense(egui::Sense::click())).clicked() {
            if course.semesters.iter().any(|v| *v) {
                course.semesters.iter_mut().for_each(|v| *v = false);
            } else {
                // todo: fix all semesters selected even with offset
                course.semesters.iter_mut().for_each(|v| *v = true);
            }
        }

        egui::ComboBox::from_id_source(ui.id().with(("lk_select", idx)))
            .width(100.0)
            .selected_text(course.exam.map(|v| format!("{v}")).unwrap_or_else(|| format!("")))
        .show_ui(ui, |ui| {
            let exams = std::iter::once(None).chain(Exam::list().iter().map(|v| Some(*v)));
            for ex in exams {
                // gray out already selected values
                ui.selectable_value(&mut course.exam, ex, ex.map(|v| format!("{v}")).unwrap_or_else(|| format!("")));
            }
        });

        ui.columns(course.semesters.len(), |col| {
            for (i, (ui, semester)) in col.iter_mut().zip(course.semesters.iter_mut()).enumerate() {
                if course.course.num_semesters.map(|v| i < (v + course.course.semester_offset)).unwrap_or(true) && i >= course.course.semester_offset {
                    if ui.add_sized((ui.available_width(), 0.0), egui::Button::new(if *semester { "x" } else { "" })).clicked() {
                        *semester = !*semester;
                    }
                }

            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
        });

        // let max_width = frame.info().window_info.size.x - 650.0;
        egui::SidePanel::right("constraint_panel").default_width(350.0).show(ctx, |ui| {
            egui::TopBottomPanel::bottom("collapsible panel").frame(egui::Frame::none()).show_inside(ui, |ui| {
                ui.add_space(10.0);
                ui.collapsing(egui::RichText::new("Hinweise").heading(), |ui| {
                    ui.label("hinweise");
                });

                ui.collapsing(egui::RichText::new("Export").heading(), |ui| {
                    ui.label("export");
                });
                ui.add_space(5.0);
            });

            ui.heading("Verpflichtungen");
            ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("rule_grid").num_columns(1).show(ui, |ui| {
                        for (name, _, is_optional) in &*self.rules.rules.borrow() {
                            let text = format!("{}{name}", if *is_optional { "(Optional) " } else { "" });
                            ui.add_enabled(false, egui::Checkbox::new(&mut false, ""));
                            ui.label(text);
                            ui.end_row();
                        }
                    });
                });
                ui.allocate_space(ui.available_size() - egui::vec2(0.0, 10.0));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            Self::draw_grid_info(ui, self.structure.num_semesters);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, field) in self.instance.as_mut().unwrap().fields.iter_mut().enumerate() {
                    Self::draw_field(ui, field, i);
                }
            });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Kurswahl", native_options, Box::new(|_| Box::new(App::new())));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "canvas",
            web_options,
            Box::new(|_| Box::new(App::new())),
        ).await.expect("failed to start eframe");
    });
}



