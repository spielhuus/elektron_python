#![allow(clippy::borrow_deref_ref)]
use lazy_static::lazy_static;
use crate::circuit::Circuit;
use crate::error::Error;
use elektron_sexp::{
     Effects, Junction, Label, LibrarySymbol, Property, SchemaElement, Stroke, Symbol, Wire,
     uuid, Bounds, Library, Schema, Shape, Transform, NoConnect,
};
use elektron_plot as plot;
use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::env::temp_dir;
use std::{fs::{File, self}, io::Read};
use uuid::Uuid;

use ndarray::{arr1, arr2, Array1, Array2};
use rand::Rng;

mod model;
mod error;
mod circuit;

const LABEL_BORDER: f64 = 2.54;

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
    ]);
}

macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string()
    };
}

macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    };
}

fn filter_properties(node: &&mut Property) -> bool {
    if let Some(effects) = &node.effects {
        !effects.hide
    } else {
        true
    }
}

fn sort_properties(a: &&mut Property, b: &&mut Property) -> std::cmp::Ordering {
    a.id.cmp(&b.id)
}

#[pyclass]
pub struct Draw {
    schema: Schema,
    libs: Library,
    last_pos: Array1<f64>,
}

#[pymethods]
impl Draw {
    #[new]
    pub fn new(library_path: Vec<String>) -> Self {
        let mut schema = Schema::new();
        schema.new_page();
        Self {
            schema,
            libs: Library::new(library_path),
            last_pos: arr1(&[10.0, 10.0]),
        }
    }

    fn add(&mut self, item: &'_ PyAny) -> PyResult<()> {
        let line: Result<model::Line, PyErr> = item.extract();
        if let Ok(line) = line {
            self.add_line(line)?;
            return Ok(());
        }
        let dot: PyResult<PyRefMut<model::Dot>> = item.extract();
        if let Ok(mut dot) = dot {
            if dot.pos == vec![0.0, 0.0] {
                dot.pos = vec![self.last_pos[0], self.last_pos[1]];
            }
            self.add_dot(&mut dot)?;
            self.last_pos = arr1(&[dot.pos[0], dot.pos[1]]);
            return Ok(());
        }
        let nc: PyResult<PyRefMut<model::Nc>> = item.extract();
        if let Ok(mut nc) = nc {
            self.add_nc(&mut nc)?;
            return Ok(());
        }
        let label: Result<model::Label, PyErr> = item.extract();
        if let Ok(mut label) = label {
            if label.pos == vec![0.0, 0.0] {
                label.pos = vec![self.last_pos[0], self.last_pos[1]];
            }
            self.add_label(label)?;
            return Ok(());
        }
        let element: Result<model::Element, PyErr> = item.extract();
        if let Ok(element) = element {
            self.add_symbol(element)?;
            return Ok(());
        }
        panic!("Item not found {:?}", item);
    }

    pub fn write(&mut self, filename: &str) -> Result<(), Error> {
        self.schema.write(filename).unwrap(); //TODO convert error
        Ok(())
    }

    #[args(netlist=false)]
    pub fn plot(
        &mut self,
        filename: Option<&str>,
        border: bool,
        scale: f64,
        imagetype: &str,
        netlist: bool,
    ) -> Result<Option<Vec<Vec<u8>>>, Error> {
        let theme = if let Ok(theme) = std::env::var("ELEKTRON_THEME") {
            theme
        } else {
            String::from("kicad_2000")
        };
        let netlist = if netlist { 
            Some(elektron_spice::Netlist::from(&self.schema).unwrap())
        } else { None };

        if let Some(filename) = filename {
            plot::plot_schema(&self.schema, None, scale, border, theme.as_str(), netlist, Some(imagetype)).unwrap();
            Ok(None)
        } else {
            /* let mut rng = rand::thread_rng();
            let num: u32 = rng.gen();
            let filename =
                String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + "." + imagetype; */
            let res = plot::plot_schema_buffer(&self.schema, scale, border, theme.as_str(), netlist, imagetype).unwrap();
            
            /* let mut f = File::open(&filename).expect("no file found");
            let metadata = fs::metadata(&filename).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read_exact(&mut buffer).expect("buffer overflow"); */
            Ok(Some(res))
            //print_from_file(&filename, &Config::default()).unwrap();
        }
    }

    pub fn circuit(&mut self, pathlist: Vec<String>) -> Circuit {
        let netlist = elektron_spice::Netlist::from(&self.schema).unwrap();
        let mut circuit = Circuit::new(String::from("draw circuit"), pathlist);
        netlist.circuit(&mut circuit.circuit).unwrap();
        circuit
    }
}

impl Draw {
    fn add_dot(&mut self, dot: &mut model::Dot) -> Result<(), Error> {
        let pos = if let (Some(atref), Some(atpin)) = (&dot.atref, &dot.atpin) {
            let pos = self.pin_pos(atref.to_string(), atpin.to_string());
            dot.pos = vec![pos[0], pos[1]];
            pos
        } else {
            self.last_pos.clone()
        };
        self.schema.push(
            0,
            SchemaElement::Junction(Junction::new(
                pos,
                uuid!(),
            )),
        )?;
        Ok(())
    }

    fn add_nc(&mut self, dot: &mut model::Nc) -> Result<(), Error> {
        let pos = if let (Some(atref), Some(atpin)) = (&dot.atref, &dot.atpin) {
            let pos = self.pin_pos(atref.to_string(), atpin.to_string());
            dot.pos = vec![pos[0], pos[1]];
            pos
        } else {
            self.last_pos.clone()
        };
        self.schema.push(
            0,
            SchemaElement::NoConnect(NoConnect::new(
                pos,
                uuid!(),
            )),
        )?;
        Ok(())
    }
    fn add_label(&mut self, label: model::Label) -> Result<(), Error> {
        let pos = self.last_pos.clone();
        let mut new_label = Label::new(
            round!(arr1(&[pos[0], pos[1]])),
            label.angle,
            label.name.as_str(),
            uuid!(),
        );
        if label.angle == 180.0 {
            new_label.effects.justify.push("right".to_string());
        } else {
            new_label.effects.justify.push("left".to_string());
        }
        self.schema.push(0, SchemaElement::Label(new_label))?;
        Ok(())
    }
    fn add_line(&mut self, line: model::Line) -> Result<(), Error> {
        let start_pos = if let Some(atdot) = line.atdot {
            arr1(&[atdot.pos[0], atdot.pos[1]])
        } else if let (Some(atpin), Some(atref)) = (line.atpin, line.atref) {
            self.pin_pos(atref, atpin)
        } else {
            self.last_pos.clone()
        };
        let end_pos = if let Some(end) = line.tox {
            arr1(&[end[0], start_pos[1]])
        } else if let Some(end) = line.toy {
            arr1(&[start_pos[0], end[1]])
        } else if let (Some(toref), Some(topin)) = (line.toxref, line.toxpin) {
            arr1(&[self.pin_pos(toref, topin)[0], start_pos[1]])
        } else if let (Some(toref), Some(topin)) = (line.toyref, line.toypin) {
            arr1(&[start_pos[0], self.pin_pos(toref, topin)[1]])
        } else {
            match line.direction {
                model::Direction::Up => arr1(&[start_pos[0], start_pos[1] - line.length]),
                model::Direction::Down => arr1(&[start_pos[0], start_pos[1] + line.length]),
                model::Direction::Left => arr1(&[start_pos[0] - line.length, start_pos[1]]),
                model::Direction::Right => arr1(&[start_pos[0] + line.length, start_pos[1]]),
            }
        };
        self.schema.push(
            0,
            SchemaElement::Wire(Wire::new(
                round!(arr2(&[
                    [start_pos[0], start_pos[1]],
                    [end_pos[0], end_pos[1]]
                ])),
                Stroke::new(),
                uuid!(),
            )),
        )?;
        self.last_pos = end_pos;
        Ok(())
    }
    fn add_symbol(&mut self, element: model::Element) -> Result<(), Error> {
        let lib_symbol = self.get_library(element.library.as_str())?;
        let sym_pin = lib_symbol.get_pin(element.pin)?;

        let pos = if let (Some(atref), Some(atpin)) = (element.atref, element.atpin) {
            self.pin_pos(atref, atpin)
        } else if let Some(dot) = element.atdot {
            arr1(&[dot.pos[0], dot.pos[1]])
        } else if let Some(pos) = element.pos {
            arr1(&[pos.0, pos.1])
        } else {
            self.last_pos.clone()
        };
        // transform pin pos
        let theta = -element.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = sym_pin.at.dot(&rot);
        verts = if let Some(mirror) = &element.mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        verts = arr1(&[pos[0], pos[1]]) - &verts;
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        if let Some(end_pos) = &element.endpos {
            let pins = lib_symbol.pins(element.unit)?;
            if pins.len() == 2 {
                let mut verts2: Array1<f64> = pins.get(element.pin as usize).unwrap().at.dot(&rot);
                verts2 = verts2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
                verts2 = arr1(&[pos[0], pos[1]]) - &verts2;
                let sym_len = verts[0] - verts2[0];
                let wire_len = ((end_pos[0] - pos[0]) - sym_len) / 2.0;
                verts = verts + arr1(&[wire_len, 0.0]);
                let mut wire1 = arr2(&[[pos[0], pos[1]], [pos[0] + wire_len, pos[1]]]);
                wire1 = wire1.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                let mut wire2 = arr2(&[
                    [pos[0] + wire_len + sym_len, pos[1]],
                    [pos[0] + 2.0 * wire_len + sym_len, pos[1]],
                ]);
                wire2 = wire2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                self.schema.push(
                    0,
                    SchemaElement::Wire(Wire::new(wire1, Stroke::new(), uuid!())),
                )?;
                self.schema.push(
                    0,
                    SchemaElement::Wire(Wire::new(wire2, Stroke::new(), uuid!())),
                )?;
                self.last_pos = arr1(&[pos[0] + 2.0 * wire_len + sym_len, pos[1]]);
            } else {
                panic!("tox and toy can only be used on symbols with two pins.")
            }
        }

        let mut symbol = Symbol::from_library(
            &lib_symbol,
            round!(verts.clone()),
            element.angle,
            element.unit,
            element.reference.to_string(),
            element.value.to_string(),
        );
        if let Some(mirror) = element.mirror {
            symbol.mirror = Some(mirror);
        }
        if let Some(properties) = element.args {
            symbol.on_schema = if let Some(on_schema) = properties.get("on_schema") {
                on_schema == "yes"
            } else {
                true
            };
            // add the extra properties
            for (k, v) in properties.into_iter() {
                if k != "on_schema" {
                    symbol.property.push(Property::new(
                        k,
                        v,
                        0,
                        round!(verts.clone()),
                        0.0,
                        Some(Effects::hidden()),
                    ));
                }
            }
        }
        self.place_property(&mut symbol).unwrap();
        self.schema.push(0, SchemaElement::Symbol(symbol))?;
        Ok(())
    }

    fn pin_pos(&self, reference: String, number: String) -> Array1<f64> {
        let symbol = self.schema.get_symbol(reference.as_str(), 1).unwrap();
        let library = self.schema.get_library(symbol.lib_id.as_str()).unwrap();
        for subsymbol in &library.symbols {
            for pin in &subsymbol.pin {
                if pin.number.0 == number {
                    //TODO: Type
                    let real_symbol = self
                        .get_symbol(reference.as_str(), subsymbol.unit as u32)
                        .unwrap();
                    return Shape::transform(real_symbol, &pin.at);
                }
            }
        }
        panic!("pin not found {}:{}", reference, number); //TODO return error
    }
    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<LibrarySymbol, Error> {
        if let Some(lib) = self.schema.get_library(name) {
            Ok(lib.clone())
        } else {
            let mut lib = self.libs.get(name).unwrap();
            if !lib.extends.is_empty() {
                let library = &name[0..name.find(':').unwrap()];
                let mut extend_symbol = self.libs.get(format!("{}:{}", library, lib.extends.as_str()).as_str())?;
                extend_symbol.property = lib.property.clone();
                for subsymbol in &mut extend_symbol.symbols {
                    let number = &subsymbol.lib_id[subsymbol.lib_id.find('_').unwrap()..subsymbol.lib_id.len()];
                    subsymbol.lib_id = format!("{}{}", lib.lib_id, number);
                }
                extend_symbol.lib_id = name.to_string();
                lib = extend_symbol;
            }
            lib.lib_id = name.to_string();
            self.schema.page(0).unwrap().libraries.push(lib.clone());
            Ok(lib)
        }
    }

    /// get the symbol by reference and unit from this schema.
    fn get_symbol(&self, reference: &str, unit: u32) -> Option<&Symbol> {
        self.schema.get_symbol(reference, unit)
    }

    fn place_property(&mut self, symbol: &mut Symbol) -> Result<(), Error> {
        let vis_field = symbol
            .property
            .iter()
            .filter_map(|node| {
                if let Some(effects) = &node.effects {
                    if !effects.hide {
                        Option::from(node)
                    } else {
                        None
                    }
                } else {
                    Option::from(node)
                }
            })
            .count();
        let lib = self.get_library(&symbol.lib_id).unwrap();

        //get and sort the shape size
        let _size = Shape::transform(symbol, &symbol.bounds(&lib).unwrap());
        let _size = if _size[[0, 0]] > _size[[1, 0]] {
            arr2(&[
                [_size[[1, 0]], _size[[0, 1]]],
                [_size[[0, 0]], _size[[1, 1]]],
            ])
        } else {
            _size
        };
        let _size = if _size[[0, 1]] > _size[[1, 1]] {
            arr2(&[
                [_size[[0, 0]], _size[[1, 1]]],
                [_size[[1, 0]], _size[[0, 1]]],
            ])
        } else {
            _size
        };
        let positions = self.pin_position(symbol, &lib);
        let mut offset = 0.0;
        let pins = lib.pins(symbol.unit)?.len();
        if pins == 1 {
            if positions[0] == 1 {
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push("left".to_string());
                        }
                        p.at = arr1(&[_size[[1, 0]] - LABEL_BORDER, symbol.at[1]]);
                        p.angle = 0.0 - symbol.angle;
                    });
                return Ok(());
            } else if positions[3] == 1 {
                //south
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], _size[[1, 1]] + LABEL_BORDER]);
                        p.angle = 0.0 - symbol.angle;
                    });
                return Ok(());
            } else if positions[2] == 1 {
                //east
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push("right".to_string());
                        }
                        p.at = arr1(&[_size[[0, 0]] - LABEL_BORDER, symbol.at[1]]);
                        p.angle = 0.0 - symbol.angle;
                    });
                return Ok(());
            } else if positions[1] == 1 {
                //south
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], _size[[0, 1]] - LABEL_BORDER]);
                        p.angle = 0.0 - symbol.angle;
                    });
                return Ok(());
            }
        } else {
            let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                _size[[0, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
            } else {
                _size[[1, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
            };
            let bottom_pos = if _size[[0, 1]] < _size[[1, 1]] {
                _size[[1, 1]] + LABEL_BORDER
            } else {
                _size[[0, 1]] + LABEL_BORDER
            };
            if positions[3] == 0 {
                //north
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], top_pos - offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[2] == 0 {
                //east
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                    - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("left"));
                        } else {
                            let mut effects = Effects::new();
                            effects.justify.push(String::from("left"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[0] == 0 {
                //west
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                    - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("right"));
                        } else {
                            let mut effects = Effects::new();
                            effects.justify.push(String::from("right"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] - LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[1] == 0 {
                //south
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], bottom_pos + offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset += LABEL_BORDER;
                    });
                return Ok(());
            }
        }
        Err(Error::ParseError)
    }

    /// get the pin position
    /// returns an array containing the number of pins:
    ///   3
    /// 2   0
    ///   1
    fn pin_position(&self, symbol: &Symbol, lib: &LibrarySymbol) -> Vec<usize> {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_shift: usize = (symbol.angle / 90.0).round() as usize;

        for pin in lib.pins(symbol.unit).unwrap() {
            let lib_pos: usize = (pin.angle / 90.0).round() as usize;
            position[lib_pos] += 1;
        }
        position.rotate_right(symbol_shift);
        if let Some(mirror) = &symbol.mirror {
            if mirror == "x" {
                position = vec![position[0], position[3], position[2], position[1]];
            } else if mirror == "y" {
                position = vec![position[2], position[1], position[0], position[3]];
            }
        }
        position
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Draw>()?;
    m.add_class::<model::Line>()?;
    m.add_class::<model::Dot>()?;
    m.add_class::<model::Label>()?;
    m.add_class::<model::Element>()?;
    m.add_class::<model::Nc>()?;
    m.add_class::<circuit::Circuit>()?;
    m.add_class::<circuit::Simulation>()?;
    Ok(())
}
