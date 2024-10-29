// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: MIT

pub mod excel_writer;
use excel_writer::ExcelSheet;

use pyo3::prelude::*;
use pyo3::types::PyList;
use rust_xlsxwriter::{Workbook};

const MAX_ROWS: u32 = 1048576;
const MAX_COLUMNS: u32 = 16384;

#[pyclass]
#[derive(Clone)]
pub struct ExcelSheetInfo {
    #[pyo3(get, set)]
    pub file_path: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub tab_color: String,
}

#[pymethods]
impl ExcelSheetInfo {
    #[new]
    fn new(file_path: String, name: String, tab_color: String) -> Self {
        ExcelSheetInfo { file_path, name: name, tab_color  }
    }
}

#[pyfunction]
fn csv_to_excel(excel_info_list: &Bound<'_, PyList>, output_file: &str) -> PyResult<()> {
    let mut workbook = Workbook::new();
    for item in excel_info_list.iter() {
        let excel_info: ExcelSheetInfo = item.extract()?;
        let mut excel_sheet = ExcelSheet::new(&mut workbook, excel_info);
        excel_sheet.write_worksheet().expect("Error writing worksheet");
    }
    workbook
        .save(output_file)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", e)))?;
    println!("Workbook saved at: {}", output_file);
    Ok(())
}

#[pymodule]
fn pyrust_xlsxwriter(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ExcelSheetInfo>()?;
    m.add_function(wrap_pyfunction!(csv_to_excel, m)?)?;
    Ok(())
}