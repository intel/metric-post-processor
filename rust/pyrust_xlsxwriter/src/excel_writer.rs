// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::BufReader;
use csv::Reader;
use pyo3::PyResult;
use rust_xlsxwriter::{Workbook, Worksheet};
use crate::{ExcelSheetInfo, MAX_COLUMNS, MAX_ROWS};

pub struct ExcelSheet<'a> {
    workbook: &'a mut Workbook,
    excel_info: ExcelSheetInfo,
}

impl<'a> ExcelSheet<'a> {
    pub fn new(workbook: &'a mut Workbook, excel_info: ExcelSheetInfo) -> Self {
        ExcelSheet { workbook, excel_info }
    }

    pub fn write_worksheet(&mut self) -> PyResult<()> {
        let worksheet = self.workbook.add_worksheet();
        let input_file = &self.excel_info.file_path;
        let file = File::open(input_file)?;
        let mut reader = Reader::from_reader(BufReader::new(file));

        println!("     importing {}...", self.excel_info.name);
        worksheet
            .set_name(&self.excel_info.name)
            .expect("Error setting worksheet name");
        worksheet.set_tab_color(&*self.excel_info.tab_color);
        let row_num = Self::write_csv_headers(worksheet, &mut reader)?;
        Self::write_csv_data(worksheet, &mut reader, row_num).expect("Error writing csv data");
        Ok(())
    }

    fn write_csv_data(worksheet: &mut Worksheet, reader: &mut Reader<BufReader<File>>, mut row_num: u32) -> PyResult<()> {
        for result in reader.records() {
            if Self::counter_exceeds_limit(row_num, MAX_ROWS, "rows", true) {
                return Ok(());
            }
            let record = result.expect("Error reading record");
            for (column, field) in record.iter().enumerate() {
                if Self::counter_exceeds_limit(column as u32, MAX_COLUMNS, "columns", false) {
                    break;
                }
                if field.is_empty() {
                    continue;
                }
                Self::write_field(worksheet, row_num, column, field);
            }
            row_num += 1;
        }
        Ok(())
    }

    fn write_field(worksheet: &mut Worksheet, row_num: u32, column: usize, field: &str) {
        match field.parse::<f64>() {
            Ok(number) => {
                if number.is_nan() {
                    return;
                }
                worksheet
                    .write_number(row_num, column as u16, number)
                    .expect("Error writing number");
            }
            Err(_) => {
                worksheet
                    .write_string(row_num, column as u16, field)
                    .expect("Error writing string");
            }
        }
    }

    fn write_csv_headers(worksheet: &mut Worksheet, reader: &mut Reader<BufReader<File>>) -> PyResult<u32> {
        let mut row_num = 0;
        if let Ok(headers) = reader.headers() {
            for (column, field) in headers.iter().enumerate() {
                if Self::counter_exceeds_limit(column as u32, MAX_COLUMNS, "columns", true) {
                    return Ok(row_num);
                }
                worksheet
                    .write_string(row_num, column as u16, field)
                    .expect("Error writing header");
            }
            row_num += 1;
        }
        Ok(row_num)
    }

    fn counter_exceeds_limit(counter: u32, counter_max: u32, counter_id: &str, verbose: bool) -> bool {
        if counter >= counter_max {
            if verbose {
                println!(
                    "     Warning: maximum number of {} ({}) exceeded", counter_id, counter_max
                );
            }
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_worksheet() {
        let mut workbook = Workbook::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "column1,column2\nvalue1,value2").unwrap();

        let excel_info = ExcelSheetInfo {
            file_path: temp_file.path().to_str().unwrap().to_string(),
            name: "Sheet1".to_string(),
            tab_color: "red".to_string(),
        };
        let mut excel_sheet = ExcelSheet::new(&mut workbook, excel_info);

        let result = excel_sheet.write_worksheet();
        assert!(result.is_ok());
    }

    #[test]
    fn test_counter_exceeds_limit() {
        assert_eq!(ExcelSheet::counter_exceeds_limit(5, 10, "test", false), false);
        assert_eq!(ExcelSheet::counter_exceeds_limit(10, 10, "test", false), true);
        assert_eq!(ExcelSheet::counter_exceeds_limit(15, 10, "test", false), true);
    }
}