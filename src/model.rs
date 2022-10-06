#![allow(clippy::borrow_deref_ref)]
use std::collections::HashMap;

use ndarray::Array1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Line {
    pub direction: Direction,
    pub length: f64,
    pub atref: Option<String>,
    pub atpin: Option<String>,
    pub atdot: Option<Dot>,
    pub toxref: Option<String>,
    pub toxpin: Option<String>,
    pub toyref: Option<String>,
    pub toypin: Option<String>,
    pub tox: Option<Array1<f64>>,
    pub toy: Option<Array1<f64>>,
}
#[pymethods]
impl Line {
    #[new]
    fn new() -> Self {
        //(Self, DrawBase) {
        // (Line { direction: String::from("left"), length: 2.54 }, DrawBase::new())
        Line {
            direction: Direction::Right,
            length: 2.54,
            atref: None,
            atpin: None,
            atdot: None,
            toxref: None,
            toxpin: None,
            toyref: None,
            toypin: None,
            tox: None,
            toy: None,
        }
    }
    pub fn up<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Up;
        slf
    }
    pub fn down<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Down;
        slf
    }
    pub fn left<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Left;
        slf
    }
    pub fn right<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Right;
        slf
    }
    pub fn length<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, len: f64) -> PyRefMut<'py, Self> {
        slf.length = len;
        slf
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.tox = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toxref = Some(reference.to_string());
                slf.toxpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", element);
    }
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.toy = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        let label: Result<Label, PyErr> = element.extract();
        if let Ok(label) = label {
            slf.toy = Some(Array1::from_vec(label.pos));
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toyref = Some(reference.to_string());
                slf.toypin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for toy: {:?}:{:?}", element, pin);
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Dot {
    pub pos: Vec<f64>,
    pub atref: Option<String>,
    pub atpin: Option<String>,
}
#[pymethods]
impl Dot {
    #[new]
    fn new() -> Self {
        Dot {
            pos: vec![0.0, 0.0],
            atref: None,
            atpin: None,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Nc {
    pub pos: Vec<f64>,
    pub atref: Option<String>,
    pub atpin: Option<String>,
}
#[pymethods]
impl Nc {
    #[new]
    fn new() -> Self {
        Self {
            pos: vec![0.0, 0.0],
            atref: None,
            atpin: None,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Label {
    pub pos: Vec<f64>,
    pub name: String,
    pub angle: f64,
}
#[pymethods]
impl Label {
    #[new]
    pub fn new(name: String) -> Self {
        Label { pos: vec![0.0, 0.0], name, angle: 0.0 }
    }
    pub fn rotate<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        angle: f64,
    ) -> PyRefMut<'py, Self> {
        slf.angle = angle;
        slf
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Element {
    pub reference: String,
    pub library: String,
    pub value: String,
    pub unit: u32,
    pub args: Option<HashMap<String, String>>,
    pub angle: f64,
    pub pin: u32,
    pub pos: Option<(f64, f64)>,
    pub atref: Option<String>,
    pub atpin: Option<String>,
    pub atdot: Option<Dot>,
    pub endpos: Option<Array1<f64>>,
    pub mirror: Option<String>,
}
#[pymethods]
impl Element {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(
        reference: String,
        library: String,
        value: String,
        unit: u32,
        kwargs: Option<&PyDict>,
    ) -> Self {
        let args = if let Some(args) = kwargs {
            let mut myargs: HashMap<String, String> = HashMap::new();
            for (k, v) in args {
                myargs.insert(k.to_string(), v.to_string());
            }
            Some(myargs)
        } else {
            None
        };
        Element {
            reference,
            library,
            value,
            unit,
            args,
            angle: 0.0,
            pin: 1,
            pos: None,
            atref: None,
            atpin: None,
            atdot: None,
            endpos: None,
            mirror: None,
        }
    }
    pub fn anchor<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, pin: u32) -> PyRefMut<'py, Self> {
        slf.pin = pin;
        slf
    }
    pub fn rotate<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        angle: f64,
    ) -> PyRefMut<'py, Self> {
        slf.angle = angle;
        slf
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        }
        let dot: Result<(f64, f64), PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.pos = Some(dot);
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.endpos = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        let label: Result<Label, PyErr> = element.extract();
        if let Ok(label) = label {
            slf.endpos = Some(Array1::from_vec(label.pos));
            return slf;
        }
        panic!("unknown type for tox: {:?}", element);
    }
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.endpos = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        let label: Result<Label, PyErr> = element.extract();
        if let Ok(label) = label {
            slf.endpos = Some(Array1::from_vec(label.pos));
            return slf;
        }
        panic!("unknown type for toy: {:?}", element);
    }
    pub fn mirror<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        mirror: String,
    ) -> PyRefMut<'py, Self> {
        slf.mirror = Some(mirror);
        slf
    }
}
