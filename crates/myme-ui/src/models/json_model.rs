use core::pin::Pin;

use cxx_qt_lib::QString;
use jsonpath_rust::JsonPath;
use serde_json::Value;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, input)]
        #[qproperty(QString, output)]
        #[qproperty(QString, jsonpath_query)]
        #[qproperty(QString, jsonpath_result)]
        #[qproperty(QString, convert_format)]
        #[qproperty(QString, converted_output)]
        #[qproperty(QString, diff_input_a)]
        #[qproperty(QString, diff_input_b)]
        #[qproperty(QString, diff_result)]
        #[qproperty(bool, is_valid)]
        #[qproperty(QString, validation_message)]
        #[qproperty(QString, error_message)]
        type JsonModel = super::JsonModelRust;

        #[qinvokable]
        fn format_json(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn minify_json(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn validate_json(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn query_jsonpath(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn convert_to_format(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn compare_json(self: Pin<&mut JsonModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut JsonModel>);
    }
}

pub struct JsonModelRust {
    input: QString,
    output: QString,
    jsonpath_query: QString,
    jsonpath_result: QString,
    convert_format: QString,
    converted_output: QString,
    diff_input_a: QString,
    diff_input_b: QString,
    diff_result: QString,
    is_valid: bool,
    validation_message: QString,
    error_message: QString,
}

impl Default for JsonModelRust {
    fn default() -> Self {
        Self {
            input: QString::from(""),
            output: QString::from(""),
            jsonpath_query: QString::from("$"),
            jsonpath_result: QString::from(""),
            convert_format: QString::from("yaml"),
            converted_output: QString::from(""),
            diff_input_a: QString::from(""),
            diff_input_b: QString::from(""),
            diff_result: QString::from(""),
            is_valid: false,
            validation_message: QString::from(""),
            error_message: QString::from(""),
        }
    }
}

impl qobject::JsonModel {
    pub fn format_json(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();

        if input.is_empty() {
            self.as_mut().set_output(QString::from(""));
            return;
        }

        match serde_json::from_str::<Value>(&input) {
            Ok(value) => {
                let formatted = serde_json::to_string_pretty(&value).unwrap_or_default();
                self.as_mut().set_output(QString::from(&formatted));
                self.as_mut().set_is_valid(true);
                self.as_mut().set_validation_message(QString::from("Valid JSON"));
            }
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Invalid JSON: {}", e)));
                self.as_mut().set_is_valid(false);
                self.as_mut().set_validation_message(QString::from(&format!("Invalid: {}", e)));
            }
        }
    }

    pub fn minify_json(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();

        if input.is_empty() {
            self.as_mut().set_output(QString::from(""));
            return;
        }

        match serde_json::from_str::<Value>(&input) {
            Ok(value) => {
                let minified = serde_json::to_string(&value).unwrap_or_default();
                self.as_mut().set_output(QString::from(&minified));
                self.as_mut().set_is_valid(true);
            }
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Invalid JSON: {}", e)));
                self.as_mut().set_is_valid(false);
            }
        }
    }

    pub fn validate_json(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();

        if input.is_empty() {
            self.as_mut().set_is_valid(false);
            self.as_mut().set_validation_message(QString::from("No input"));
            return;
        }

        match serde_json::from_str::<Value>(&input) {
            Ok(value) => {
                self.as_mut().set_is_valid(true);

                // Count some stats
                let (objects, arrays, strings, numbers, bools, nulls) = Self::count_types(&value);
                let msg = format!(
                    "Valid JSON - Objects: {}, Arrays: {}, Strings: {}, Numbers: {}, Booleans: {}, Nulls: {}",
                    objects, arrays, strings, numbers, bools, nulls
                );
                self.as_mut().set_validation_message(QString::from(&msg));
            }
            Err(e) => {
                self.as_mut().set_is_valid(false);
                self.as_mut().set_validation_message(QString::from(&format!("Invalid: {}", e)));
            }
        }
    }

    fn count_types(value: &Value) -> (usize, usize, usize, usize, usize, usize) {
        let mut objects = 0;
        let mut arrays = 0;
        let mut strings = 0;
        let mut numbers = 0;
        let mut bools = 0;
        let mut nulls = 0;

        fn count(
            v: &Value,
            o: &mut usize,
            a: &mut usize,
            s: &mut usize,
            n: &mut usize,
            b: &mut usize,
            nl: &mut usize,
        ) {
            match v {
                Value::Object(map) => {
                    *o += 1;
                    for val in map.values() {
                        count(val, o, a, s, n, b, nl);
                    }
                }
                Value::Array(arr) => {
                    *a += 1;
                    for val in arr {
                        count(val, o, a, s, n, b, nl);
                    }
                }
                Value::String(_) => *s += 1,
                Value::Number(_) => *n += 1,
                Value::Bool(_) => *b += 1,
                Value::Null => *nl += 1,
            }
        }

        count(value, &mut objects, &mut arrays, &mut strings, &mut numbers, &mut bools, &mut nulls);
        (objects, arrays, strings, numbers, bools, nulls)
    }

    pub fn query_jsonpath(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();
        let query = self.as_ref().jsonpath_query().to_string();

        if input.is_empty() {
            self.as_mut().set_jsonpath_result(QString::from(""));
            return;
        }

        let value: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Invalid JSON: {}", e)));
                return;
            }
        };

        let path = match JsonPath::try_from(query.as_str()) {
            Ok(p) => p,
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Invalid JSONPath: {}", e)));
                return;
            }
        };

        let results = path.find(&value);
        let result_str = serde_json::to_string_pretty(&results).unwrap_or_default();
        self.as_mut().set_jsonpath_result(QString::from(&result_str));
    }

    pub fn convert_to_format(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();
        let format = self.as_ref().convert_format().to_string();

        if input.is_empty() {
            self.as_mut().set_converted_output(QString::from(""));
            return;
        }

        let value: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Invalid JSON: {}", e)));
                return;
            }
        };

        let converted = match format.as_str() {
            "yaml" => match serde_yaml::to_string(&value) {
                Ok(s) => s,
                Err(e) => {
                    self.as_mut()
                        .set_error_message(QString::from(&format!("YAML conversion error: {}", e)));
                    return;
                }
            },
            "toml" => {
                // TOML requires a table at the root
                if !value.is_object() {
                    self.as_mut().set_error_message(QString::from(
                        "TOML requires an object at the root level",
                    ));
                    return;
                }
                match toml::to_string_pretty(&value) {
                    Ok(s) => s,
                    Err(e) => {
                        self.as_mut().set_error_message(QString::from(&format!(
                            "TOML conversion error: {}",
                            e
                        )));
                        return;
                    }
                }
            }
            _ => {
                self.as_mut().set_error_message(QString::from("Unknown format"));
                return;
            }
        };

        self.as_mut().set_converted_output(QString::from(&converted));
    }

    pub fn compare_json(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input_a = self.as_ref().diff_input_a().to_string();
        let input_b = self.as_ref().diff_input_b().to_string();

        if input_a.is_empty() || input_b.is_empty() {
            self.as_mut().set_diff_result(QString::from(""));
            return;
        }

        let value_a: Value = match serde_json::from_str(&input_a) {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("JSON A invalid: {}", e)));
                return;
            }
        };

        let value_b: Value = match serde_json::from_str(&input_b) {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("JSON B invalid: {}", e)));
                return;
            }
        };

        if value_a == value_b {
            self.as_mut()
                .set_diff_result(QString::from("identical:The JSON documents are identical"));
        } else {
            // Simple diff - show the differences
            let diff = Self::generate_diff(&value_a, &value_b, "");
            self.as_mut().set_diff_result(QString::from(&format!(
                "different:{}",
                if diff.is_empty() { "Documents differ (structural differences)" } else { &diff }
            )));
        }
    }

    fn generate_diff(a: &Value, b: &Value, path: &str) -> String {
        let mut diffs = Vec::new();

        match (a, b) {
            (Value::Object(map_a), Value::Object(map_b)) => {
                // Check for keys in A but not in B
                for key in map_a.keys() {
                    let new_path =
                        if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };

                    if !map_b.contains_key(key) {
                        diffs.push(format!("- {}: removed", new_path));
                    } else {
                        let sub_diff = Self::generate_diff(&map_a[key], &map_b[key], &new_path);
                        if !sub_diff.is_empty() {
                            diffs.push(sub_diff);
                        }
                    }
                }

                // Check for keys in B but not in A
                for key in map_b.keys() {
                    if !map_a.contains_key(key) {
                        let new_path =
                            if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                        diffs.push(format!("+ {}: added", new_path));
                    }
                }
            }
            (Value::Array(arr_a), Value::Array(arr_b)) => {
                if arr_a.len() != arr_b.len() {
                    diffs.push(format!(
                        "~ {}: array length {} -> {}",
                        if path.is_empty() { "root" } else { path },
                        arr_a.len(),
                        arr_b.len()
                    ));
                }
            }
            _ => {
                if a != b {
                    diffs.push(format!(
                        "~ {}: {} -> {}",
                        if path.is_empty() { "root" } else { path },
                        serde_json::to_string(a).unwrap_or_default(),
                        serde_json::to_string(b).unwrap_or_default()
                    ));
                }
            }
        }

        diffs.join("\n")
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input(QString::from(""));
        self.as_mut().set_output(QString::from(""));
        self.as_mut().set_jsonpath_query(QString::from("$"));
        self.as_mut().set_jsonpath_result(QString::from(""));
        self.as_mut().set_converted_output(QString::from(""));
        self.as_mut().set_diff_input_a(QString::from(""));
        self.as_mut().set_diff_input_b(QString::from(""));
        self.as_mut().set_diff_result(QString::from(""));
        self.as_mut().set_is_valid(false);
        self.as_mut().set_validation_message(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
