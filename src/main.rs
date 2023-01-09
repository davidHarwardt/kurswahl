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

#[derive(Debug, Clone)]
struct Structure<T> {
    fields: Vec<Field<T>>,
    num_semesters: usize,
}

impl Structure<Course> {
    fn new(fields: Vec<Field<Course>>, num_semesters: usize) -> Self {
        Self { fields, num_semesters }
    }

    fn as_instance(self) -> Structure<CourseInstance> {
        let fields = self.fields.into_iter().map(|v| v.as_instance(self.num_semesters)).collect();
        Structure {
            fields,
            num_semesters: self.num_semesters,
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

#[derive(Debug, Clone)]
struct Course {
    name: String,
    id: String,
    tags: HashSet<String>,
}

#[derive(Debug)]
struct CourseInstance {
    course: Course,
    semesters: Vec<bool>,
}

impl Course {
    fn new<S: Into<String>>(name: impl Into<String>, id: impl Into<String>, tags: Vec<S>) -> Self {
        let name = name.into();
        let id = id.into();
        let tags = tags.into_iter().map(|v| v.into()).collect();
        Self { name, tags, id }
    }

    fn as_instance(self, num_semesters: usize) -> CourseInstance {
        let semesters = (0..num_semesters).map(|_| false).collect();
        CourseInstance {
            course: self,
            semesters,
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
            Course::new::<&str>("Sport-Theorie", "spo-th", vec![]),
        ]),

        Field::new_max_usable("Ensemblemusik", vec![
            Course::new::<&str>("Chor", "chor", vec![]),
            Course::new::<&str>("Bläser", "blaeser", vec![]),
        ], 2),
        Field::new_max_usable("Zusatzkurse", vec![
            Course::new::<&str>("CCC", "ccc", vec![]),
            Course::new::<&str>("Debating", "debating", vec![]),
            Course::new::<&str>("Digitale Welten", "dw", vec![]),
        ], 2),
        Field::new_max_usable("Seminarkurse", vec![
            Course::new::<&str>("Neurowissenschaften", "neuro", vec![]),
            Course::new::<&str>("Doping", "doping", vec![]),
            Course::new::<&str>("Finanzmathematik", "fin-mat", vec![]),
        ], 2),
        Field::new("Sport", vec![
            Course::new::<&str>("Ski und Snowboard", "ski", vec![]),
        ]),
    ], 4);
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

struct App {

}

impl App {
    fn new() -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("constraint_panel").show(ctx, |ui| {
            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("1. AF");
            ui.group(|ui| {
                egui::Grid::new("task_grid").num_columns(3).show(ui, |ui| {
                    ui.label("Deutsch");
                    // egui::ComboBox::from_label();
                    ui.button("1. LK").clicked();
                    ui.columns(4, |col| {
                        for ui in col {
                            ui.add_sized((ui.available_width(), 0.0), egui::Button::new("--x--"));
                        }
                    });
                    ui.end_row();

                });
            });
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("kurswahl", native_options, Box::new(|_| Box::new(App::new())));
}



